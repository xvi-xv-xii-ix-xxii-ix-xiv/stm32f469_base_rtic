//! # Blue LED Control Utilities
//!
//! Provides timing constants and state management for blue LED operations.

use crate::peripherals::blue_led::BlueLed;
use crate::peripherals::traits::GpioPin;

/// LED timing constants (milliseconds)
pub const LED_ON_DURATION: u32 = 4_000; // Active state duration
pub const LED_OFF_DURATION: u32 = 1_000; // Inactive state duration
pub const LED_CHECK_INTERVAL: u32 = 60_000; // Status check interval

/// LED operational states
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LedState {
    /// LED is actively illuminated
    Active,
    /// LED is in inactive state
    Inactive,
}

impl LedState {
    /// Gets duration for current state
    pub fn duration(&self) -> u32 {
        match self {
            LedState::Active => LED_ON_DURATION,
            LedState::Inactive => LED_OFF_DURATION,
        }
    }
}

/// Toggles LED state and returns new duration
///
/// # Arguments
/// * `led` - Mutable reference to BlueLed instance
///
/// # Returns
/// Duration in milliseconds for new state
///
/// # Example
/// ```rust
/// let mut led = BlueLed::init_off(pk3);
/// let duration = toggle_led(&mut led);
/// ```
pub fn toggle_led(led: &mut BlueLed) -> u32 {
    let current_state = match led.is_set_high() {
        Ok(true) => LedState::Inactive,
        Ok(false) => LedState::Active,
        Err(_) => {
            #[cfg(feature = "debug")]
            defmt::warn!("LED state check failed");
            LedState::Inactive // Default to safe state
        }
    };

    match current_state {
        LedState::Active => {
            if let Err(e) = led.set_high() {
                #[cfg(feature = "debug")]
                defmt::error!("Failed to deactivate LED: {:?}", e);
            }
            LedState::Inactive.duration()
        }
        LedState::Inactive => {
            if let Err(e) = led.set_low() {
                #[cfg(feature = "debug")]
                defmt::error!("Failed to activate LED: {:?}", e);
            }
            LedState::Active.duration()
        }
    }
}
