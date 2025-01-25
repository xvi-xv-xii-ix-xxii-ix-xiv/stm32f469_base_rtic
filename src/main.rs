//! # STM32F469 Firmware
//!
//! This is the main firmware application for the STM32F469 microcontroller implementing
//! a real-time system using RTIC (Real-Time Interrupt-driven Concurrency) framework.
//!
//! ## Key Features
//! - USB 2.0 Full Speed communication with OTG controller
//! - UART communication via USART6 with DMA transfers
//! - Dual LED status indication system (blue operational status, red error reporting)
//! - Thread-safe ring buffers for data management
//! - Comprehensive error handling with persistent error codes
//! - Low-power idle mode with interrupt wakeup
//!
//! ## Hardware Requirements
//! - STM32F469NI-Discovery board
//! - Blue LED connected on PK3 (LD4 on discovery board)
//! - Red LED connected on PD5 (LD5 on discovery board)
//! - USART6 peripheral using:
//!   - TX: PG14 (connected to external UART converter)
//!   - RX: PG9 (connected to external UART converter)
//! - USB OTG FS port configured in device mode
//!
//! ## Architecture Overview
//! The application follows these design principles:
//! - RTIC framework for concurrency management
//! - Separation of hardware abstraction layers (HAL) and application logic
//! - Non-blocking DMA-driven data transfers
//! - Atomic resource sharing between tasks
//! - Error queue system with visual feedback
//!
//! ## Task Priorities
//! | Task                  | Priority | Description                              |
//! |-----------------------|----------|------------------------------------------|
//! | USB Handling          | 4        | Highest priority for USB communication   |
//! | DMA Stream Handlers   | 3        | Data transfer completion handling        |
//! | USART6 Handler        | 3        | Serial communication management          |
//! | Error Display         | 5        | Critical error visualization             |
//! | LED Status            | 1        | Lowest priority for status indication    |
//!
//! ## Safety Considerations
//! - All shared resources use RTIC's mutex protection
//! - Critical sections keep interrupts disabled <100 cycles
//! - DMA transfers use hardware-verified buffer boundaries
//! - Error states trigger failsafe LED patterns

#![no_main]
#![no_std]

#[cfg(feature = "debug")]
use defmt_rtt as _; // Global logger for RTT-based debugging

#[cfg(feature = "debug")]
mod debug; // Debug utilities (RTT initialization, formatted logging)

#[cfg(feature = "debug")]
use debug::{init as debug_init, log_error};

#[cfg(feature = "debug")]
use panic_probe as _; // Panic handler with defmt integration

#[cfg(not(feature = "debug"))]
use panic_halt as _; // Production panic handler (system freeze)

mod config; // System constants and clock configuration
mod data_structures; // Circular buffers and data containers
mod errors; // Error type definitions and conversions
mod macros; // Procedural macros for code generation
mod peripherals; // Hardware abstraction layer implementation
mod task_handlers; // RTIC task implementations
mod utils; // Helper functions and utilities

use crate::errors::errors::{DeviceError, UsbError};
use crate::task_handlers::error_handlers::add_error_code;
use rtic::app;
use rtic_monotonics::systick::prelude::*;

// System timer configuration: 1ms timebase using SysTick
systick_monotonic!(Mono, 1000);

#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {
    use super::*;
    use crate::config::{SYSCLK};
    use crate::peripherals::stm32f469_init::init_peripherals;
    use crate::peripherals::traits::GpioPin;
    use crate::task_handlers::blue_led::{toggle_led, LED_CHECK_INTERVAL};
    use crate::task_handlers::dma2::{handle_dma_rx, handle_dma_tx, handle_usart_error};
    use crate::task_handlers::error_handlers::{has_errors};
    use crate::task_handlers::otg_fs::{handle_usb, process_rx_buffer};
    use crate::task_handlers::red_led_handler::update_red_led;

    /// Shared system resources protected by RTIC mutexes
    #[shared]
    struct Shared {
        blue_led: peripherals::blue_led::BlueLed, // Status LED controller
        red_led: peripherals::red_led::RedLed,    // Error LED controller
        usart_6: peripherals::usart_6::Usart6Controller, // UART interface with DMA
        otg_fs: peripherals::otg_fs::OtgFsController<'static>, // USB device controller
        is_red_led_active: bool,                  // Error display state flag
        is_blue_led_blinking: bool,               // Normal operation indicator flag
        ring_buffer_rx: data_structures::ring_buffer::RingBuffer, // Incoming data buffer
        ring_buffer_tx: data_structures::ring_buffer::RingBuffer, // Outgoing data buffer
    }

    /// Local task-specific resources (unshared state)
    #[local]
    struct Local {
        retry_count: u8, // Counter for communication retries
    }

    /// System initialization routine
    ///
    /// # Safety
    /// - Must be first function executed after reset
    /// - Configures all critical hardware peripherals
    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        #[cfg(feature = "debug")]
        debug_init(); // Initialize debug channel if enabled

        let peripherals = init_peripherals(ctx.device)
            .expect("Peripheral initialization failed - check hardware configuration");

        // Configure monotonic timer for async delays
        Mono::start(ctx.core.SYST, SYSCLK);

        // Spawn persistent background tasks
        blue_led_blink::spawn().ok();
        task_display_error_codes::spawn().ok();

        #[cfg(feature = "debug")]
        debug_print!("System initialized at {} Hz", SYSCLK);

        (
            Shared {
                blue_led: peripherals.blue_led,
                red_led: peripherals.red_led,
                usart_6: peripherals.usart_6,
                otg_fs: peripherals.otg_fs,
                is_red_led_active: false,
                is_blue_led_blinking: true,
                ring_buffer_rx: data_structures::ring_buffer::RingBuffer::new(),
                ring_buffer_tx: data_structures::ring_buffer::RingBuffer::new(),
            },
            Local { retry_count: 0 },
        )
    }

    /// Idle task - enters low-power sleep mode
    ///
    /// # Behavior
    /// - Runs with lowest priority when no tasks are active
    /// - Uses WFI instruction to minimize power consumption
    /// - Wakeup occurs via interrupt triggers
    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        #[cfg(feature = "debug")]
        debug_print!("Entering low-power idle mode");

        loop {
            cortex_m::asm::wfi();
        }
    }

    /// USART6 interrupt handler
    ///
    /// # Responsibilities
    /// - Handle DMA transfer completion events
    /// - Manage UART error conditions
    /// - Trigger data processing tasks
    #[task(binds = USART6, shared = [usart_6, ring_buffer_rx], local = [retry_count], priority = 3)]
    fn usart6(mut ctx: usart6::Context) {
        #[cfg(feature = "debug")]
        defmt::info!("USART6 IRQ: Checking DMA state");

        ctx.shared.usart_6.lock(|usart| {
            ctx.shared.ring_buffer_rx.lock(|rx| {
                match usart.is_dma_rx_is_idle() {
                    Ok(true) => match handle_dma_rx(usart, rx) {
                        Err(e) => {
                            #[cfg(feature = "debug")]
                            defmt::warn!("DMA RX error: {:?}", e);
                            handle_error(e.into());
                        }
                        Ok(()) => {
                            #[cfg(feature = "debug")]
                            defmt::debug!("Spawning buffer processing task");
                            ring_buffer_rx_to_serial::spawn().ok();
                        }
                    },
                    Ok(false) => {
                        #[cfg(feature = "debug")]
                        defmt::trace!("DMA RX active - no action");
                    }
                    Err(e) => {
                        #[cfg(feature = "debug")]
                        defmt::error!("DMA state check failed: {:?}", e);
                        handle_error(e.into());
                    }
                }

                if let Err(e) = handle_usart_error(usart, ctx.local.retry_count) {
                    #[cfg(feature = "debug")]
                    defmt::warn!("USART error: {:?}", e);
                    handle_error(e.into());
                }
            });
        });
    }

    /// DMA2 Stream6 (TX) interrupt handler
    ///
    /// # Behavior
    /// - Clears transfer complete flag
    /// - Does NOT restart transfers automatically (handled by tasks)
    #[task(binds = DMA2_STREAM6, shared = [usart_6], priority = 3)]
    fn dma2_stream6(mut ctx: dma2_stream6::Context) {
        #[cfg(feature = "debug")]
        defmt::trace!("DMA2 Stream6 (TX) complete");

        ctx.shared.usart_6.lock(|usart| {
            usart.clear_dma_tx_complete_flag();
        });
    }

    /// DMA2 Stream1 (RX) interrupt handler
    ///
    /// # Responsibilities
    /// - Handle incoming data from UART RX DMA
    /// - Trigger buffer processing task
    #[task(binds = DMA2_STREAM1, shared = [usart_6, ring_buffer_rx], priority = 3)]
    fn dma2_stream1(mut ctx: dma2_stream1::Context) {
        #[cfg(feature = "debug")]
        defmt::debug!("DMA2 Stream1 (RX) complete");

        ctx.shared.usart_6.lock(|usart| {
            ctx.shared.ring_buffer_rx.lock(|rx| {
                if let Err(e) = handle_dma_rx(usart, rx) {
                    handle_error(e.into());
                }
                ring_buffer_rx_to_serial::spawn().ok();
            });
        });
    }

    /// USB OTG FS interrupt handler
    ///
    /// # Behavior
    /// - Handles USB enumeration and configuration
    /// - Manages USB data transfers to/from TX buffer
    /// - Triggers UART forwarding when data received
    #[task(binds = OTG_FS, shared = [otg_fs, ring_buffer_tx], priority = 4)]
    fn otg_fs(mut ctx: otg_fs::Context) {
        ctx.shared.otg_fs.lock(|usb| {
            if !usb.poll() {
                handle_error(UsbError::PollError.into());
                return;
            }

            if usb.is_configured() {
                ctx.shared
                    .ring_buffer_tx
                    .lock(|tx| match handle_usb(usb, tx) {
                        Ok(bytes_processed) => {
                            #[cfg(feature = "debug")]
                            defmt::info!("USB processed {} bytes", bytes_processed);
                            if bytes_processed > 0 {
                                ring_buffer_tx_to_usart_dma::spawn(bytes_processed).ok();
                            }
                        }
                        Err(e) => {
                            handle_error(e.into());
                        }
                    });
            } else {
                #[cfg(feature = "debug")]
                defmt::warn!("USB not configured");
            }
        });
    }

    /// Process RX buffer and send to USB serial
    ///
    /// # Execution Context
    /// - Triggered by DMA completion or USART idle detection
    /// - Runs as async task to allow non-blocking operation
    #[task(shared = [otg_fs, ring_buffer_rx], priority = 3)]
    async fn ring_buffer_rx_to_serial(mut ctx: ring_buffer_rx_to_serial::Context) {
        #[cfg(feature = "debug")]
        defmt::debug!("Processing RX buffer");

        ctx.shared.otg_fs.lock(|usb| {
            ctx.shared.ring_buffer_rx.lock(|rx| {
                if let Err(e) = process_rx_buffer(usb, rx) {
                    handle_error(e.into());
                }
            });
        });
    }

    /// Transmit TX buffer contents via UART DMA
    ///
    /// # Parameters
    /// - `bytes_processed`: Number of bytes to transmit from buffer
    #[task(shared = [usart_6, ring_buffer_tx], priority = 3)]
    async fn ring_buffer_tx_to_usart_dma(
        mut ctx: ring_buffer_tx_to_usart_dma::Context,
        bytes_processed: usize,
    ) {
        #[cfg(feature = "debug")]
        defmt::debug!("TX DMA starting with {} bytes", bytes_processed);

        ctx.shared.usart_6.lock(|usart| {
            ctx.shared.ring_buffer_tx.lock(|tx| {
                if let Err(e) = handle_dma_tx(usart, tx, bytes_processed) {
                    handle_error(e.into());
                }
            });
        });
    }

    /// Blue LED status indication task
    ///
    /// # Behavior Patterns
    /// - Normal operation: 1Hz blink
    /// - Error active: Solid off
    /// - Manual override: Solid on
    #[task(shared = [blue_led, is_red_led_active, is_blue_led_blinking], priority = 1)]
    async fn blue_led_blink(mut ctx: blue_led_blink::Context) {
        loop {
            let delay = ctx.shared.blue_led.lock(|led| {
                if ctx.shared.is_red_led_active.lock(|active| *active) {
                    if let Err(e) = led.set_high() {
                        handle_error(e.into());
                    }
                    LED_CHECK_INTERVAL
                } else if ctx.shared.is_blue_led_blinking.lock(|blinking| *blinking) {
                    toggle_led(led)
                } else {
                    if let Err(e) = led.set_high() {
                        handle_error(e.into());
                    }
                    LED_CHECK_INTERVAL
                }
            });

            Mono::delay(delay.millis()).await;
        }
    }

    /// Error code visualization task
    ///
    /// # Display Protocol
    /// - Short blink: Digit separator
    /// - Long blink: Error code digit (quantity = digit value)
    /// - 500ms pause between codes
    #[task(shared = [red_led, is_red_led_active], priority = 5)]
    async fn task_display_error_codes(mut ctx: task_display_error_codes::Context) {
        let mut buffer = [0u8; 100];

        loop {
            if !has_errors() {
                ctx.shared.is_red_led_active.lock(|active| *active = false);
                Mono::delay(500.millis()).await;
                continue;
            }

            ctx.shared.is_red_led_active.lock(|active| *active = true);

            let current_time = Mono::now().ticks();
            ctx.shared.red_led.lock(|red_led| {
                update_red_led(red_led, current_time, &mut buffer);
            });

            ctx.shared.is_red_led_active.lock(|active| *active = false);
        }
    }
}

/// Central error handling facility
///
/// # Error Handling Flow
/// 1. Log error to debug output (if enabled)
/// 2. Add error code to persistent queue
/// 3. Trigger error visualization task
fn handle_error(error: DeviceError) {
    #[cfg(feature = "debug")]
    log_error(error.description());

    if add_error_code(error.code()).is_err() {
        #[cfg(feature = "debug")]
        defmt::error!("Error queue overflow - code: {}", error.code());
    }
}
