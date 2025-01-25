//! # USB OTG FS Communication Handler
//!
//! This module provides functionality for managing USB data transfers between
//! the microcontroller and host device. Key features include:
//! - Bidirectional data transfer handling
//! - Buffer management with error recovery
//! - Partial write handling with data preservation

use crate::config::DATA_PACKET_SIZE;
use crate::data_structures::ring_buffer::RingBuffer;
use crate::errors::errors::{DeviceError, UsbError};
use crate::peripherals::otg_fs::OtgFsController;

/// Handles USB communication lifecycle
///
/// # Arguments
/// * `usb` - USB controller instance
/// * `tx` - Transmit ring buffer
///
/// # Returns
/// - `Ok(bytes_processed)` - Number of bytes successfully processed
/// - `Err(DeviceError)` - Encountered error during processing
///
/// # Flow
/// 1. Checks USB configuration status
/// 2. Processes incoming USB data
/// 3. Returns transfer metrics
pub fn handle_usb(
    usb: &mut OtgFsController<'static>,
    tx: &mut RingBuffer,
) -> Result<usize, DeviceError> {
    if !usb.is_configured() {
        #[cfg(feature = "debug")]
        defmt::warn!("USB device not configured - skipping transfer");
        return Ok(0);
    }

    let bytes_processed = process_usb_data(usb, tx)?;
    Ok(bytes_processed)
}

/// Processes incoming USB data to transmit buffer
///
/// # Arguments
/// * `usb` - USB controller instance
/// * `tx` - Transmit ring buffer
///
/// # Errors
/// Returns `DeviceError` on:
/// - USB read failures
/// - Buffer overflow conditions
fn process_usb_data(
    usb: &mut OtgFsController<'static>,
    tx: &mut RingBuffer,
) -> Result<usize, DeviceError> {
    match usb.read() {
        Ok(Some((data, count))) => {
            #[cfg(feature = "debug")]
            defmt::debug!("USB RX: {} bytes", count);

            if tx.available_space() < count {
                #[cfg(feature = "debug")]
                defmt::error!("TX buffer overflow: {} > {}", count, tx.available_space());
                return Err(DeviceError::from(UsbError::BufferOverflow));
            }

            tx.push(&data[..count])
                .map_err(|_| DeviceError::from(UsbError::BufferOverflow))?;
            Ok(count)
        }
        Ok(None) => {
            #[cfg(feature = "debug")]
            defmt::trace!("No USB data available");
            Ok(0)
        }
        Err(e) => {
            #[cfg(feature = "debug")]
            defmt::error!("USB read failure: {:?}", e);
            Err(e.into())
        }
    }
}

/// Transmits data from receive buffer via USB
///
/// # Arguments
/// * `usb` - USB controller instance
/// * `rx` - Receive ring buffer
///
/// # Returns
/// - `Ok(bytes_sent)` - Total bytes successfully transmitted
/// - `Err(DeviceError)` - Transmission failure
///
/// # Behavior
/// - Handles partial writes by preserving unsent data
/// - Manages buffer state during retries
pub fn process_rx_buffer(
    usb: &mut OtgFsController<'static>,
    rx: &mut RingBuffer,
) -> Result<usize, DeviceError> {
    let mut tx_buffer = [0u8; DATA_PACKET_SIZE];
    let mut total_sent = 0;

    if rx.is_empty() {
        #[cfg(feature = "debug")]
        defmt::trace!("RX buffer empty - nothing to transmit");
        return Ok(0);
    }

    let bytes_read = rx.pop(&mut tx_buffer);
    #[cfg(feature = "debug")]
    defmt::debug!("Preparing to send {} bytes", bytes_read);

    match usb.write(&tx_buffer[..bytes_read]) {
        Ok(written) => {
            total_sent += written;

            if written < bytes_read {
                #[cfg(feature = "debug")]
                defmt::warn!("Partial write: {}/{} bytes", written, bytes_read);

                let remaining = &tx_buffer[written..bytes_read];
                rx.push(remaining).map_err(|_| {
                    #[cfg(feature = "debug")]
                    defmt::error!("Failed to preserve {} unsent bytes", remaining.len());
                    DeviceError::from(UsbError::BufferOverflow)
                })?;
            }
        }
        Err(e) => {
            #[cfg(feature = "debug")]
            defmt::error!("USB write failure: {:?}", e);
            return Err(e.into());
        }
    }

    #[cfg(feature = "debug")]
    defmt::info!("Total transmitted: {} bytes", total_sent);

    Ok(total_sent)
}