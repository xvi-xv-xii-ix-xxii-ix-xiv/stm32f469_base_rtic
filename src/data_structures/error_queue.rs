use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use heapless::spsc::Queue;

/// A channel for transmitting errors, protected by a Mutex.
///
/// The `ERROR_QUEUE` is a statically allocated, single-producer, single-consumer (SPSC) queue
/// for error codes of type `u16`. The queue can hold up to 256 error codes at a time.
/// The queue is protected by a `Mutex` to ensure safe access across interrupts and other contexts.
pub static ERROR_QUEUE: Mutex<RefCell<Queue<u16, 256>>> = Mutex::new(RefCell::new(Queue::new()));
