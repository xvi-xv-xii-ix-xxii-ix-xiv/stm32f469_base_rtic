//! # Fixed-Size Ring Buffer Implementation
//!
//! Provides a no-std compatible circular buffer with:
//! - Constant-time operations
//! - Thread-unsafe but interrupt-safe design
//! - Configurable buffer size
//! - Detailed error handling

use crate::config::RING_BUFFER_LEN;
use crate::errors::errors::RingBufferError;
use core::fmt;
use heapless::Vec;

/// Circular buffer for USART communication
pub struct RingBuffer {
    buffer: [u8; RING_BUFFER_LEN],
    write_pos: usize,
    read_pos: usize,
    count: usize,
}

impl RingBuffer {
    /// Creates new empty buffer
    #[inline]
    pub const fn new() -> Self {
        Self {
            buffer: [0u8; RING_BUFFER_LEN],
            write_pos: 0,
            read_pos: 0,
            count: 0,
        }
    }

    /// Appends data to buffer
    ///
    /// # Errors
    /// Returns `RingBufferError::BufferOverflow` if insufficient space
    pub fn push(&mut self, data: &[u8]) -> Result<(), RingBufferError> {
        let data_len = data.len();
        if data_len > self.available_space() {
            #[cfg(feature = "debug")]
            defmt::warn!(
                "Buffer overflow attempt: {} > {}",
                data_len,
                self.available_space()
            );
            return Err(RingBufferError::BufferOverflow);
        }

        let first_chunk_len = core::cmp::min(data_len, RING_BUFFER_LEN - self.write_pos);
        let second_chunk_len = data_len - first_chunk_len;

        // Copy data in 1 or 2 operations
        self.buffer[self.write_pos..self.write_pos + first_chunk_len]
            .copy_from_slice(&data[..first_chunk_len]);

        if second_chunk_len > 0 {
            self.buffer[..second_chunk_len].copy_from_slice(&data[first_chunk_len..]);
        }

        self.write_pos = (self.write_pos + data_len) % RING_BUFFER_LEN;
        self.count += data_len;

        #[cfg(feature = "debug")]
        defmt::debug!("Pushed {} bytes. New count: {}", data_len, self.count);

        Ok(())
    }

    /// Appends data from heapless::Vec
    ///
    /// # Errors
    /// Returns `RingBufferError::BufferOverflow` if insufficient space
    pub fn push_n<const N: usize>(&mut self, data: &Vec<u8, N>) -> Result<(), RingBufferError> {
        self.push(data.as_slice())
    }

    /// Removes data from buffer into slice
    ///
    /// # Returns
    /// Number of bytes actually read
    pub fn pop(&mut self, output: &mut [u8]) -> usize {
        let to_read = core::cmp::min(output.len(), self.count);
        if to_read == 0 {
            return 0;
        }

        let first_chunk_len = core::cmp::min(to_read, RING_BUFFER_LEN - self.read_pos);
        let second_chunk_len = to_read - first_chunk_len;

        output[..first_chunk_len]
            .copy_from_slice(&self.buffer[self.read_pos..self.read_pos + first_chunk_len]);

        if second_chunk_len > 0 {
            output[first_chunk_len..to_read].copy_from_slice(&self.buffer[..second_chunk_len]);
        }

        self.read_pos = (self.read_pos + to_read) % RING_BUFFER_LEN;
        self.count -= to_read;

        #[cfg(feature = "debug")]
        defmt::debug!("Popped {} bytes. Remaining: {}", to_read, self.count);

        to_read
    }

    /// Extracts bytes into heapless::Vec
    pub fn pop_n<const N: usize>(&mut self, count: usize) -> Vec<u8, N> {
        let mut result = Vec::new();
        let to_read = core::cmp::min(count, self.count).min(N);

        if to_read == 0 {
            return result;
        }

        let mut temp_buf = [0u8; RING_BUFFER_LEN];
        let bytes_read = self.pop(&mut temp_buf[..to_read]);

        if result.extend_from_slice(&temp_buf[..bytes_read]).is_err() {
            #[cfg(feature = "debug")]
            defmt::error!("Failed to populate result vector");
        }

        result
    }

    /// Gets current data count
    #[inline]
    pub const fn len(&self) -> usize {
        self.count
    }

    /// Checks if buffer is empty
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Calculates available space
    #[inline]
    pub const fn available_space(&self) -> usize {
        RING_BUFFER_LEN - self.count
    }

    /// Clears buffer contents and zeros memory
    pub fn clear(&mut self) {
        self.write_pos = 0;
        self.read_pos = 0;
        self.count = 0;
        self.buffer.iter_mut().for_each(|x| *x = 0);

        #[cfg(feature = "debug")]
        defmt::info!("Buffer cleared and zeroized");
    }
}

impl Default for RingBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Automatically clear buffer when dropped
impl Drop for RingBuffer {
    fn drop(&mut self) {
        self.clear();
        #[cfg(feature = "debug")]
        defmt::trace!("RingBuffer dropped and cleared");
    }
}

/// Debug implementation showing key metrics
impl fmt::Debug for RingBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RingBuffer[used: {}/{}]", self.count, RING_BUFFER_LEN)
    }
}
