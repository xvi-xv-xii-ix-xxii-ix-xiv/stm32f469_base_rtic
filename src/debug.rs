#[allow(unused_imports)]
use defmt_rtt as _; // Important! This initializes RTT

/// Global logger configuration
use defmt::*;

/// Custom timestamp function for defmt logging
///
/// This function generates a unique timestamp for each log entry. It uses an
/// `AtomicU32` counter that is incremented with each call to `timestamp()`.
/// This timestamp is used by `defmt` for precise log timing.
#[export_name = "_defmt_timestamp"]
fn timestamp() -> u64 {
    use core::sync::atomic::{AtomicU32, Ordering};

    static COUNT: AtomicU32 = AtomicU32::new(0);

    // Increment and return the timestamp as a 64-bit value
    COUNT.fetch_add(1, Ordering::Relaxed) as u64
}

/// Initializes the logging system and sends test messages
///
/// This function sends test log messages at various log levels (trace, debug,
/// info, warn, error) to verify that the RTT logger is initialized and working
/// correctly. These messages can help during development to ensure the logging
/// system is functioning properly.
pub fn init() {
    // Sending test messages to check RTT functionality
    trace!("Trace: RTT initialized");
    debug!("Debug: RTT initialized");
    info!("Info: RTT initialized successfully");
    warn!("Warning: Test message");
    error!("Error: Test message");
}

/// Custom panic handler
///
/// This function is called when a panic occurs. It logs an error message
/// and then triggers an undefined instruction (`udf`) to halt the program.
/// This allows for a controlled stop during a panic situation, ensuring
/// that the system doesn't continue executing in an invalid state.
#[defmt::panic_handler]
fn panic() -> ! {
    error!("Panic occurred!");
    cortex_m::asm::udf() // Trigger a system halt
}

/// Global error handler
///
/// This function logs a custom error message using the `defmt` error log level.
/// It provides a centralized way to report errors in the system.
pub fn log_error(msg: &str) {
    error!("Error occurred: {}", msg);
}
