//! # Red LED Controller with Morse Code Support
//!
//! This module provides:
//! - Basic LED control
//! - Morse code signaling capabilities
//! - State machine for code transmission
//! - Timing management

use crate::config::MAX_MORSE_LENGTH;
use crate::utils::morse::number_to_morse;
use stm32f4xx_hal::gpio::{gpiod::PD5, Output, PushPull};

/// Morse code transmission states
#[derive(Debug, Clone, Copy)]
pub enum MorseState {
    /// No active transmission
    Idle,
    /// Currently transmitting a symbol
    Signal,
    /// Pause between symbols
    Pause,
}

/// Red LED controller with Morse code capabilities
pub struct RedLed {
    pin: PD5<Output<PushPull>>,
    pub(crate) morse_sequence: Option<[u8; MAX_MORSE_LENGTH]>,
    pub(crate) morse_length: usize,
    pub(crate) morse_index: usize,
    pub(crate) morse_state: MorseState,
    pub(crate) last_toggle: u32,
}

impl RedLed {
    /// Initializes LED in OFF state
    ///
    /// # Arguments
    /// * `pin` - PD5 pin in push-pull output mode
    pub fn init_off(mut pin: PD5<Output<PushPull>>) -> Self {
        pin.set_high();
        RedLed {
            pin,
            morse_sequence: None,
            morse_length: 0,
            morse_index: 0,
            morse_state: MorseState::Idle,
            last_toggle: 0,
        }
    }

    /// Starts new Morse code sequence
    ///
    /// # Arguments
    /// * `code` - Numeric code to convert to Morse
    /// * `buffer` - Temporary conversion buffer
    ///
    /// # Errors
    /// Returns error if:
    /// - Code conversion fails
    /// - Resulting sequence exceeds MAX_MORSE_LENGTH
    pub fn start_morse_sequence(
        &mut self,
        code: u16,
        buffer: &mut [u8],
    ) -> Result<(), &'static str> {
        let length = number_to_morse(code, buffer).map_err(|_| "Conversion failed")?;

        if length > MAX_MORSE_LENGTH {
            return Err("Sequence too long");
        }

        let mut sequence = [0u8; MAX_MORSE_LENGTH];
        sequence[..length].copy_from_slice(&buffer[..length]);

        self.morse_sequence = Some(sequence);
        self.morse_length = length;
        self.morse_index = 0;
        self.morse_state = MorseState::Idle;
        self.last_toggle = 0;

        #[cfg(feature = "debug")]
        defmt::debug!("Morse sequence started: {:?}", &sequence[..length]);

        Ok(())
    }

    /// Resets Morse code transmission state
    pub fn reset_morse_state(&mut self) {
        self.morse_sequence = None;
        self.morse_length = 0;
        self.morse_index = 0;
        self.morse_state = MorseState::Idle;
        self.last_toggle = 0;

        #[cfg(feature = "debug")]
        defmt::trace!("Morse state reset");
    }

    /// Gets current Morse symbol from sequence
    ///
    /// # Returns
    /// - `Some(char)` if sequence is active and index valid
    /// - `None` if sequence completed or invalid
    pub fn current_symbol(&self) -> Option<char> {
        self.morse_sequence
            .as_ref()
            .and_then(|seq| seq.get(self.morse_index))
            .map(|&b| b as char)
    }

    /// Sets LED to OFF state
    pub fn set_high(&mut self) {
        self.pin.set_high();
    }

    /// Sets LED to ON state
    pub fn set_low(&mut self) {
        self.pin.set_low();
    }

    /// Checks if LED is currently ON
    pub fn is_on(&self) -> bool {
        self.pin.is_set_low()
    }

    /// Toggles LED state
    pub fn toggle(&mut self) {
        if self.is_on() {
            self.set_high();
        } else {
            self.set_low();
        }
    }
}