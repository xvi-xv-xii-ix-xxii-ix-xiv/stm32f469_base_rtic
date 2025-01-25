//! # DMA2 Stream Management
//!
//! Handles DMA operations for USART6 communication including:
//! - Error recovery mechanisms
//! - Data transfer between ring buffers and DMA
//! - Retry logic for failed operations

use crate::config::{DMA_BUFFER_LEN, RING_BUFFER_LEN};
use crate::data_structures::ring_buffer::RingBuffer;
use crate::errors::errors::{DmaError, UsartError};
use crate::peripherals::usart_6::{Usart6Controller, UsartFlag};

/// Maximum retry attempts for DMA operations
pub const MAX_RETRY_COUNT: u8 = 3;

/// Handles USART-related DMA errors with recovery logic
pub fn handle_usart_error(
    usart: &mut Usart6Controller,
    retry_count: &mut u8,
) -> Result<(), DmaError> {
    if usart.check_dma_rx_error().unwrap_or(false) {
        handle_error_condition(usart, retry_count, |u| u.restart_dma_rx())?;
    }

    if usart.check_dma_tx_error().unwrap_or(false) {
        handle_error_condition(usart, retry_count, |u| u.restart_dma_tx())?;
    }

    usart.clear_usart_flags(UsartFlag::RXNE);
    Ok(())
}

/// Processes DMA TX operations
pub fn handle_dma_tx(
    usart: &mut Usart6Controller,
    tx: &mut RingBuffer,
    bytes_processed: usize,
) -> Result<(), DmaError> {
    let mut buffer = [0u8; DMA_BUFFER_LEN];
    let data = prepare_tx_data(tx, bytes_processed, &mut buffer)?;
    transfer_to_dma(usart, data)?;
    usart.clear_dma_tx_complete_flag();
    Ok(())
}

/// Processes DMA RX operations with full error handling
pub fn handle_dma_rx(usart: &mut Usart6Controller, rx: &mut RingBuffer) -> Result<(), DmaError> {
    // Process received data
    let mut buffer = [0u8; DMA_BUFFER_LEN];
    let data = read_from_dma(usart, &mut buffer)?;
    store_to_buffer(rx, data)?;
    usart.clear_dma_rx_complete_flag();

    Ok(())
}

// Shared error handling logic
fn handle_error_condition<F>(
    usart: &mut Usart6Controller,
    retry_count: &mut u8,
    restart_fn: F,
) -> Result<(), DmaError>
where
    F: FnOnce(&mut Usart6Controller) -> Result<(), UsartError>,
{
    *retry_count = retry_count.saturating_add(1);

    if *retry_count > MAX_RETRY_COUNT {
        *retry_count = 0;
        return Err(DmaError::RetryLimitExceeded);
    }

    usart.clear_errors();
    restart_fn(usart).map_err(|_| DmaError::InitError)?;
    Ok(())
}

// TX data preparation with static buffer
fn prepare_tx_data<'a>(
    tx: &mut RingBuffer,
    bytes_processed: usize,
    buffer: &'a mut [u8; DMA_BUFFER_LEN],
) -> Result<&'a [u8], DmaError> {
    if bytes_processed > DMA_BUFFER_LEN || tx.len() < bytes_processed {
        return Err(DmaError::BufferUnderflow);
    }

    let data = tx.pop_n::<RING_BUFFER_LEN>(bytes_processed);
    buffer[..bytes_processed].copy_from_slice(&data);
    Ok(&buffer[..bytes_processed])
}

// DMA write operation
fn transfer_to_dma(usart: &mut Usart6Controller, data: &[u8]) -> Result<(), DmaError> {
    let buffer = usart
        .get_tx_buffer_slice(data.len())
        .ok_or(DmaError::WriteError)?;

    buffer.copy_from_slice(data);

    usart.write_dma().map_err(|_| {
        usart.clear_errors();
        DmaError::WriteError
    })
}

// DMA read operation with buffer management
fn read_from_dma<'a>(
    usart: &mut Usart6Controller,
    buffer: &'a mut [u8; DMA_BUFFER_LEN],
) -> Result<&'a [u8], DmaError> {
    // Initiate DMA read operation
    usart.read_dma().map_err(|_| {
        usart.clear_errors();
        DmaError::ReadError
    })?;

    // Clear USART flags after successful read
    usart.clear_usart_flags(UsartFlag::RXNE);

    let bytes_received =
        DMA_BUFFER_LEN - usart.get_dma_rx_length().map_err(|_| DmaError::ReadError)?;

    let data = usart
        .get_rx_buffer_slice(bytes_received)
        .ok_or(DmaError::ReadError)?;

    buffer[..bytes_received].copy_from_slice(data);
    Ok(&buffer[..bytes_received])
}

// Buffer storage with overflow protection
fn store_to_buffer(rx: &mut RingBuffer, data: &[u8]) -> Result<(), DmaError> {
    rx.push(data).map_err(|_| DmaError::BufferOverflow)
}
