//! # Blue LED Controller
//!
//! This module provides control for the blue status LED (PK3) with:
//! - State tracking
//! - Safe GPIO operations
//! - Error handling
//! - Debug display implementation

use crate::errors::errors::LedError;
use crate::peripherals::traits::GpioPin;
use core::fmt;
use stm32f4xx_hal::gpio::{gpiok::PK3, Output, PushPull};

/// Blue LED controller with state tracking
#[derive(Debug)]
pub struct BlueLed {
    pin: PK3<Output<PushPull>>,
    state: bool,
}

impl BlueLed {
    /// Creates BlueLed instance with LED initially ON
    ///
    /// # Arguments
    /// * `pin` - PK3 pin in push-pull output mode
    pub fn init_on(mut pin: PK3<Output<PushPull>>) -> Self {
        pin.set_low();
        BlueLed { pin, state: true }
    }

    /// Creates BlueLed instance with LED initially OFF
    ///
    /// # Arguments
    /// * `pin` - PK3 pin in push-pull output mode
    pub fn init_off(mut pin: PK3<Output<PushPull>>) -> Self {
        pin.set_high();
        BlueLed { pin, state: false }
    }

    /// Gets current LED state
    pub fn state(&self) -> bool {
        self.state
    }
}

/// GPIO Pin trait implementation
impl GpioPin for BlueLed {
    type Error = LedError;

    /// Sets LED to high state (OFF)
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.pin.set_high();
        self.state = false;
        Ok(())
    }

    /// Sets LED to low state (ON)
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.pin.set_low();
        self.state = true;
        Ok(())
    }

    /// Checks if LED is in high state
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(self.pin.is_set_high())
    }

    /// Toggles LED state
    fn toggle(&mut self) -> Result<(), Self::Error> {
        if self.state {
            self.set_low()
        } else {
            self.set_high()
        }
    }
}

/// Debug display implementation
impl fmt::Display for BlueLed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Blue LED: {}", if self.state { "ON" } else { "OFF" })
    }
}
