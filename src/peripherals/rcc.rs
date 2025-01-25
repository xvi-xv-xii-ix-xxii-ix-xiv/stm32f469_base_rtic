//! # RCC (Reset and Clock Control) Configuration
//!
//! This module provides clock configuration management for STM32F4 microcontrollers.
//! It handles system clock sources and peripheral clock distribution with:
//! - HSE (High Speed External) clock support
//! - SYSCLK, PCLK1, and PCLK2 frequency configuration
//! - PLL48CLK requirement for USB functionality
//!
//! ## Safety Considerations
//! - Clock configuration should be performed once at system startup
//! - Incorrect clock settings may cause unstable operation

use stm32f4xx_hal::{
    pac,
    prelude::*,
    rcc::{Clocks, RccExt},
};

/// Clock configuration structure
pub struct RccConfig {
    /// Final clock configuration results
    pub clocks: Clocks,
}

impl RccConfig {
    /// Creates new clock configuration
    ///
    /// # Arguments
    /// * `device` - RCC peripheral instance
    /// * `hse` - External oscillator frequency in Hz
    /// * `sysclk` - Target system clock frequency in Hz
    /// * `pclk1` - APB1 peripheral clock frequency in Hz
    /// * `pclk2` - APB2 peripheral clock frequency in Hz
    ///
    /// # Example
    /// ```rust
    /// let dp = pac::Peripherals::take().unwrap();
    /// let rcc_config = RccConfig::new(
    ///     dp.RCC,
    ///     8_000_000,  // 8 MHz HSE crystal
    ///     168_000_000, // 168 MHz SYSCLK
    ///     42_000_000,  // 42 MHz PCLK1
    ///     84_000_000,  // 84 MHz PCLK2
    /// );
    /// ```
    ///
    /// # Note
    /// The PLL48CLK is automatically configured for USB operation
    pub fn new(device: pac::RCC, hse: u32, sysclk: u32, pclk1: u32, pclk2: u32) -> Self {
        let rcc = device.constrain();
        let cfgr = rcc
            .cfgr
            .use_hse(hse.Hz())
            .sysclk(sysclk.Hz())
            .pclk1(pclk1.Hz())
            .pclk2(pclk2.Hz())
            .require_pll48clk();

        let clocks = cfgr.freeze();

        RccConfig { clocks }
    }
}
