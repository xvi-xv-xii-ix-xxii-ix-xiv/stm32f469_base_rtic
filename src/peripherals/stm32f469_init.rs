//! # STM32F469 Peripheral Initialization
//!
//! This module handles the initialization of critical peripherals for the STM32F469 microcontroller.
//! It configures:
//! - Clock tree through RCC
//! - GPIO pins for LEDs and communication interfaces
//! - USART6 for serial communication
//! - USB OTG FS for USB device functionality
//! - Interrupt configuration for peripherals
//!
//! ## Safety Considerations
//! - Must be called only once during system startup
//! - Direct hardware access requires proper sequencing
//! - Interrupt masks should match actual peripheral usage

use crate::config::{HSE, PCLK1, PCLK2, SYSCLK};
use crate::errors::errors::InitError;
use crate::peripherals::blue_led::BlueLed;
use crate::peripherals::otg_fs::OtgFsController;
use crate::peripherals::rcc::RccConfig;
use crate::peripherals::red_led::RedLed;
use crate::peripherals::usart_6::Usart6Controller;
use cortex_m::singleton;
use stm32f4xx_hal::pac::Interrupt;
use stm32f4xx_hal::{pac, prelude::*};

/// Container for initialized hardware peripherals
pub struct InitializedPeripherals {
    /// Blue LED controller (PK3)
    pub blue_led: BlueLed,
    /// Red LED controller (PD5)
    pub red_led: RedLed,
    /// USART6 controller with DMA capabilities
    pub usart_6: Usart6Controller,
    /// USB OTG FS device controller
    pub otg_fs: OtgFsController<'static>,
}

/// Initializes all critical system peripherals
///
/// # Parameters
/// * `device` - Peripheral access crate structure
///
/// # Errors
/// Returns `InitError` if:
/// - Clock configuration fails
/// - USART6 initialization fails
/// - USB initialization fails
///
/// # Safety
/// - Must maintain exclusive access to hardware resources
/// - Interrupt configuration must match actual usage
pub fn init_peripherals(device: pac::Peripherals) -> Result<InitializedPeripherals, InitError> {
    #[allow(non_snake_case)]
    let pac::Peripherals {
        RCC,
        GPIOA,
        GPIOD,
        GPIOK,
        GPIOG,
        USART6,
        DMA2,
        OTG_FS_DEVICE,
        OTG_FS_GLOBAL,
        OTG_FS_PWRCLK,
        ..
    } = device;

    // ===================== Clock Configuration =====================
    let rcc_config: &'static mut RccConfig = singleton!(
        : RccConfig = RccConfig::new(RCC, HSE, SYSCLK, PCLK1, PCLK2)
    )
    .ok_or(InitError::RccError)?;

    // ===================== LED Initialization =====================
    // Blue LED (PK3) - System status indicator
    let gpiok = GPIOK.split();
    let blue_led = BlueLed::init_off(gpiok.pk3.into_push_pull_output());

    // Red LED (PD5) - Error state indicator
    let gpiod = GPIOD.split();
    let red_led = RedLed::init_off(gpiod.pd5.into_push_pull_output());

    // ===================== USART6 Configuration =====================
    let gpiog = GPIOG.split();
    let usart6 = Usart6Controller::init(
        USART6,
        DMA2,
        gpiog.pg14.into_alternate::<8>(), // TX pin
        gpiog.pg9.into_alternate::<8>(),  // RX pin
        rcc_config,
    )
    .map_err(|_| InitError::UsartError)?;

    // ===================== USB OTG FS Configuration =====================
    let gpioa = GPIOA.split();
    let otg_fs = OtgFsController::new(
        OTG_FS_GLOBAL,
        OTG_FS_DEVICE,
        OTG_FS_PWRCLK,
        gpioa.pa11.into_alternate::<10>(), // DM pin
        gpioa.pa12.into_alternate::<10>(), // DP pin
        rcc_config,
    )
    .map_err(|_| InitError::UsbError)?;

    // ===================== Interrupt Configuration =====================
    // SAFETY: Single unmask operations during initialization
    unsafe {
        cortex_m::peripheral::NVIC::unmask(Interrupt::OTG_FS);
        cortex_m::peripheral::NVIC::unmask(Interrupt::USART6);
        cortex_m::peripheral::NVIC::unmask(Interrupt::DMA2_STREAM1);
        cortex_m::peripheral::NVIC::unmask(Interrupt::DMA2_STREAM6);
    }

    Ok(InitializedPeripherals {
        blue_led,
        red_led,
        usart_6: usart6,
        otg_fs,
    })
}
