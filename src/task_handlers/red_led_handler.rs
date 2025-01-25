//! # Red LED Morse Code Handler
//!
//! Implements error code visualization using Morse code patterns on the red LED.
//! Supports:
//! - Dot (.) and dash (-) symbols
//! - Inter-symbol and inter-word spacing
//! - Error code queuing system

use crate::peripherals::red_led::{MorseState, RedLed};
use crate::task_handlers::error_handlers::get_first_error_code;

/// Morse code timing constants (milliseconds)
pub const MORSE_DOT_DURATION: u32 = 200; // Duration of a dot (ms)
pub const MORSE_DASH_DURATION: u32 = MORSE_DOT_DURATION * 3; // Duration of a dash
pub const MORSE_SYMBOL_PAUSE: u32 = MORSE_DOT_DURATION; // Pause between symbols
pub const MORSE_WORD_PAUSE: u32 = MORSE_DOT_DURATION * 7; // Pause between words

/// Updates LED state based on Morse code timing and error codes
///
/// # Arguments
/// * `led` - Red LED controller
/// * `current_time` - System timestamp in milliseconds
/// * `buffer` - Temporary buffer for Morse code conversion
///
/// # State Machine
/// 1. Idle: Waiting to start new symbol
/// 2. Signal: LED active (dot/dash transmission)
/// 3. Pause: LED inactive (space between symbols/words)
pub fn update_red_led(led: &mut RedLed, current_time: u32, buffer: &mut [u8]) {
    match led.morse_sequence {
        Some(_) => handle_active_sequence(led, current_time),
        None => start_new_sequence(led, buffer),
    }
}

/// Handles ongoing Morse code transmission
fn handle_active_sequence(led: &mut RedLed, current_time: u32) {
    let elapsed = calculate_elapsed(current_time, led.last_toggle);

    match led.morse_state {
        MorseState::Idle => process_idle_state(led, current_time),
        MorseState::Signal => process_signal_state(led, elapsed),
        MorseState::Pause => process_pause_state(led, elapsed, current_time),
    }
}

/// Starts new Morse sequence from error queue
fn start_new_sequence(led: &mut RedLed, buffer: &mut [u8]) {
    if let Some(code) = get_first_error_code() {
        if let Err(e) = led.start_morse_sequence(code, buffer) {
            #[cfg(feature = "debug")]
            defmt::error!("Morse init failed: {:?}", e);
        }
    } else {
        #[cfg(feature = "debug")]
        defmt::trace!("No error codes in queue");
    }
}

/// Processes IDLE state (waiting to start symbol)
fn process_idle_state(led: &mut RedLed, current_time: u32) {
    if led.morse_index >= led.morse_length {
        led.reset_morse_state();
        return;
    }

    if let Some(symbol) = led.current_symbol() {
        match symbol {
            '.' | '-' => activate_led(led, current_time),
            ' ' => start_pause(led, current_time),
            _ => handle_invalid_symbol(led),
        }
    }
}

/// Processes SIGNAL state (LED active)
fn process_signal_state(led: &mut RedLed, elapsed: u32) {
    let duration = match led.current_symbol() {
        Some('.') => MORSE_DOT_DURATION,
        Some('-') => MORSE_DASH_DURATION,
        _ => return led.reset_morse_state(),
    };

    if elapsed >= duration {
        deactivate_led(led);
    }
}

/// Processes PAUSE state (LED inactive)
fn process_pause_state(led: &mut RedLed, elapsed: u32, current_time: u32) {
    let required_pause = match led.current_symbol() {
        Some('.') | Some('-') => MORSE_SYMBOL_PAUSE,
        Some(' ') => MORSE_WORD_PAUSE,
        _ => return led.reset_morse_state(),
    };

    if elapsed >= required_pause {
        advance_sequence(led, current_time);
    }
}

/// Calculates elapsed time with overflow protection
fn calculate_elapsed(current: u32, last: u32) -> u32 {
    current.wrapping_sub(last)
}

/// Activates LED and updates state
fn activate_led(led: &mut RedLed, timestamp: u32) {
    led.set_low();
    led.morse_state = MorseState::Signal;
    led.last_toggle = timestamp;
}

/// Deactivates LED and starts pause
fn deactivate_led(led: &mut RedLed) {
    led.set_high();
    led.morse_state = MorseState::Pause;
}

/// Advances to next symbol in sequence
fn advance_sequence(led: &mut RedLed, timestamp: u32) {
    led.morse_index += 1;
    led.morse_state = MorseState::Idle;
    led.last_toggle = timestamp;
}

/// Starts inter-symbol pause
fn start_pause(led: &mut RedLed, timestamp: u32) {
    led.morse_state = MorseState::Pause;
    led.last_toggle = timestamp;
}

/// Handles invalid Morse symbols
fn handle_invalid_symbol(led: &mut RedLed) {
    #[cfg(feature = "debug")]
    defmt::warn!("Invalid Morse symbol detected");

    led.reset_morse_state();
}