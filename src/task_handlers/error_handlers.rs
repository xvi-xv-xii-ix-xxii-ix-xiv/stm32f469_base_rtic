use crate::data_structures::error_queue::ERROR_QUEUE;
use cortex_m::interrupt::{self};

/// Adds an error code to the queue.
///
/// # Parameters:
/// - `code`: The error code to enqueue.
///
/// # Returns:
/// - `Ok(())` if the error code was successfully added.
/// - `Err("Error queue is full")` if the queue is full.
pub fn add_error_code(code: u16) -> Result<(), &'static str> {
    interrupt::free(|cs| {
        let mut queue = ERROR_QUEUE.borrow(cs).borrow_mut();
        if queue.enqueue(code).is_err() {
            Err("Error queue is full")
        } else {
            Ok(())
        }
    })
}

/// Retrieves the first error code from the queue.
///
/// # Returns:
/// - `Some(u16)` if an error code is available.
/// - `None` if the queue is empty.
pub fn get_first_error_code() -> Option<u16> {
    interrupt::free(|cs| {
        let mut queue = ERROR_QUEUE.borrow(cs).borrow_mut();
        queue.dequeue()
    })
}

/// Checks if the error queue contains any errors.
///
/// # Returns:
/// - `true` if the queue is not empty.
/// - `false` if the queue is empty.
pub fn has_errors() -> bool {
    interrupt::free(|cs| {
        let queue = ERROR_QUEUE.borrow(cs).borrow();
        !queue.is_empty()
    })
}
