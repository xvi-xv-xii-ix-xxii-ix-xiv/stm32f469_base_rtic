//! # USART6 Controller Implementation
//!
//! This module provides DMA-driven UART communication handling for USART6 peripheral
//! on STM32F469 microcontrollers. Key features include:
//! - Full-duplex DMA transfers with configurable buffers
//! - Error detection and recovery mechanisms
//! - Hardware flag management for USART status
//! - Thread-safe buffer access patterns
//!
//! ## Hardware Configuration
//! - Uses PG14 (TX) and PG9 (RX) pins in alternate function mode 8
//! - Requires DMA2 streams 6 (TX) and 1 (RX)
//! - Baud rate configured in `config` module
//!
//! ## Safety Considerations
//! - DMA buffer access protected by singleton pattern
//! - Atomic flag checks for transfer status
//! - Automatic error recovery for DMA faults

use stm32f4xx_hal::{
    dma::{DmaFlag, StreamsTuple, Transfer},
    gpio::{
        gpiog::{PG14, PG9},
        Alternate,
    },
    pac::{DMA2, USART6},
    prelude::*,
    serial::{Config, Serial},
};

use crate::config::{DMA_BUFFER_LEN, USART6_BAUD_RATE};
use crate::data_structures::typedefs;
use crate::dma_cfg;
use crate::errors::errors::UsartError;
use crate::peripherals::rcc::RccConfig;

use bitflags::bitflags;

bitflags! {
    /// USART status flags for interrupt handling
    pub struct UsartFlag: u32 {
        const RXNE = 1 << 5;  // Receive Data Register Not Empty
        const TXE  = 1 << 7;  // Transmit Data Register Empty
        const TC   = 1 << 6;  // Transmission Complete
    }
}

/// Main controller for USART6 peripheral with DMA capabilities
pub struct Usart6Controller {
    dma_tx: Option<typedefs::DmaTxTransfer>,
    dma_rx: Option<typedefs::DmaRxTransfer>,
    tx_buffer: &'static mut [u8],
    rx_buffer: &'static mut [u8],
}

impl Usart6Controller {
    /// Initializes USART6 peripheral with DMA configuration
    ///
    /// # Arguments
    /// * `usart_6` - USART6 peripheral instance
    /// * `dma_2` - DMA2 controller instance
    /// * `tx_pin` - Configured TX pin (PG14)
    /// * `rx_pin` - Configured RX pin (PG9)
    /// * `clocks` - System clock configuration
    ///
    /// # Errors
    /// Returns `UsartError::NotInitialized` if:
    /// - Serial port initialization fails
    /// - DMA buffer allocation fails
    ///
    /// # Safety
    /// - Must be called only once during system initialization
    /// - Requires exclusive access to DMA2 streams
    pub fn init(
        usart_6: USART6,
        dma_2: DMA2,
        tx_pin: PG14<Alternate<8>>,
        rx_pin: PG9<Alternate<8>>,
        clocks: &RccConfig,
    ) -> Result<Self, UsartError> {
        let serial = Serial::new(
            usart_6,
            (tx_pin, rx_pin),
            Config {
                baudrate: USART6_BAUD_RATE.bps(),
                wordlength: stm32f4xx_hal::serial::config::WordLength::DataBits8,
                parity: stm32f4xx_hal::serial::config::Parity::ParityNone,
                stopbits: stm32f4xx_hal::serial::config::StopBits::STOP1,
                dma: stm32f4xx_hal::serial::config::DmaConfig::TxRx,
                ..Default::default()
            },
            &clocks.clocks,
        )
        .map_err(|_| UsartError::NotInitialized)?;

        let streams = StreamsTuple::new(dma_2);
        let (tx, mut rx) = serial.split();

        // Allocate DMA buffers using cortex_m singleton
        let tx_buffer = cortex_m::singleton!(: [u8; DMA_BUFFER_LEN] = [0; DMA_BUFFER_LEN])
            .ok_or(UsartError::NotInitialized)?;
        let rx_buffer = cortex_m::singleton!(: [u8; DMA_BUFFER_LEN] = [0; DMA_BUFFER_LEN])
            .ok_or(UsartError::NotInitialized)?;

        // SAFETY: Buffer pointers remain valid for 'static lifetime
        let tx_buffer_dma = unsafe { &mut *(tx_buffer as *mut [u8]) };
        let rx_buffer_dma = unsafe { &mut *(rx_buffer as *mut [u8]) };

        rx.listen_idle();
        let usart = unsafe { &*USART6::ptr() };
        usart
            .cr1()
            .modify(|_, w| w.txeie().clear_bit().tcie().clear_bit());

        let mut dma_tx =
            Transfer::init_memory_to_peripheral(streams.6, tx, tx_buffer_dma, None, dma_cfg!());
        let dma_rx =
            Transfer::init_peripheral_to_memory(streams.1, rx, rx_buffer_dma, None, dma_cfg!());

        dma_tx.start(|_tx| {});

        #[cfg(feature = "debug")]
        defmt::info!("USART6 initialized successfully");

        Ok(Self {
            dma_tx: Some(dma_tx),
            dma_rx: Some(dma_rx),
            tx_buffer,
            rx_buffer,
        })
    }

    /// Starts DMA transmission
    ///
    /// # Errors
    /// Returns `UsartError::NotInitialized` if DMA TX not configured
    pub fn start_dma_tx(&mut self) -> Result<(), UsartError> {
        self.dma_tx
            .as_mut()
            .ok_or(UsartError::NotInitialized)?
            .start(|_| ());

        #[cfg(feature = "debug")]
        defmt::debug!("DMA TX started");
        Ok(())
    }

    /// Starts DMA reception
    ///
    /// # Errors
    /// Returns `UsartError::NotInitialized` if DMA RX not configured
    pub fn start_dma_rx(&mut self) -> Result<(), UsartError> {
        self.dma_rx
            .as_mut()
            .ok_or(UsartError::NotInitialized)?
            .start(|_| ());

        #[cfg(feature = "debug")]
        defmt::debug!("DMA RX started");
        Ok(())
    }

    /// Restarts DMA reception with error recovery
    ///
    /// # Flow
    /// 1. Clear previous transfer errors
    /// 2. Reinitialize DMA transfer
    ///
    /// # Errors
    /// Returns `UsartError::NotInitialized` if DMA RX not configured
    pub fn restart_dma_rx(&mut self) -> Result<(), UsartError> {
        let dma = self.dma_rx.as_mut().ok_or(UsartError::NotInitialized)?;
        dma.clear_transfer_error();
        dma.start(|_| {});

        #[cfg(feature = "debug")]
        defmt::warn!("DMA RX restarted");
        Ok(())
    }

    /// Restarts DMA transmission with error recovery
    ///
    /// # Flow
    /// 1. Clear previous transfer errors
    /// 2. Reinitialize DMA transfer
    ///
    /// # Errors
    /// Returns `UsartError::NotInitialized` if DMA TX not configured
    pub fn restart_dma_tx(&mut self) -> Result<(), UsartError> {
        let dma = self.dma_tx.as_mut().ok_or(UsartError::NotInitialized)?;
        dma.clear_transfer_error();
        dma.start(|_| {});

        #[cfg(feature = "debug")]
        defmt::warn!("DMA TX restarted");
        Ok(())
    }

    /// Initiates DMA write transfer
    ///
    /// # Example
    /// ```rust
    /// usart.write_dma()?;
    /// ```
    ///
    /// # Errors
    /// Propagates errors from restart_dma_tx
    pub fn write_dma(&mut self) -> Result<(), UsartError> {
        self.restart_dma_tx()?;
        #[cfg(feature = "debug")]
        defmt::trace!("DMA write started");
        Ok(())
    }

    /// Initiates DMA read transfer
    ///
    /// # Example
    /// ```rust
    /// usart.read_dma()?;
    /// ```
    ///
    /// # Errors
    /// Propagates errors from restart_dma_rx
    pub fn read_dma(&mut self) -> Result<(), UsartError> {
        self.restart_dma_rx()?;
        #[cfg(feature = "debug")]
        defmt::trace!("DMA read started");
        Ok(())
    }

    /// Checks for DMA RX transfer errors and automatically restarts
    ///
    /// # Returns
    /// - `Ok(true)` if error was detected and handled
    /// - `Ok(false)` if no errors present
    /// - `Err(UsartError)` if initialization check fails
    pub fn check_dma_rx_error(&mut self) -> Result<bool, UsartError> {
        let has_error = self
            .dma_rx
            .as_ref()
            .ok_or(UsartError::NotInitialized)?
            .is_transfer_error();

        #[cfg(feature = "debug")]
        if has_error {
            defmt::error!("DMA RX error detected");
            self.restart_dma_rx()?;
        }

        Ok(has_error)
    }

    /// Checks for DMA TX transfer errors and automatically restarts
    ///
    /// # Returns
    /// - `Ok(true)` if error was detected and handled
    /// - `Ok(false)` if no errors present
    /// - `Err(UsartError)` if initialization check fails
    pub fn check_dma_tx_error(&mut self) -> Result<bool, UsartError> {
        let has_error = self
            .dma_tx
            .as_ref()
            .ok_or(UsartError::NotInitialized)?
            .is_transfer_error();

        #[cfg(feature = "debug")]
        if has_error {
            defmt::error!("DMA TX error detected");
            self.restart_dma_tx()?;
        }

        Ok(has_error)
    }

    /// Checks DMA TX completion status
    ///
    /// # Errors
    /// Returns `UsartError::NotInitialized` if DMA TX not configured
    pub fn is_dma_tx_complete(&self) -> Result<bool, UsartError> {
        self.dma_tx
            .as_ref()
            .ok_or(UsartError::NotInitialized)
            .map(|dma| dma.is_transfer_complete())
    }

    /// Checks DMA RX completion status
    ///
    /// # Errors
    /// Returns `UsartError::NotInitialized` if DMA RX not configured
    pub fn is_dma_rx_complete(&self) -> Result<bool, UsartError> {
        self.dma_rx
            .as_ref()
            .ok_or(UsartError::NotInitialized)
            .map(|dma| dma.is_transfer_complete())
    }

    /// Gets read-only slice of RX buffer
    ///
    /// # Parameters
    /// - `length`: Maximum bytes to return (clamped to buffer size)
    ///
    /// # Returns
    /// `Some(&[u8])` if buffer initialized, `None` otherwise
    pub fn get_rx_buffer_slice(&self, length: usize) -> Option<&[u8]> {
        (!self.rx_buffer.is_empty()).then(|| &self.rx_buffer[..length.min(self.rx_buffer.len())])
    }

    /// Gets mutable slice of TX buffer
    ///
    /// # Parameters
    /// - `length`: Maximum bytes to return (clamped to buffer size)
    ///
    /// # Returns
    /// `Some(&mut [u8])` if buffer initialized, `None` otherwise
    pub fn get_tx_buffer_slice(&mut self, length: usize) -> Option<&mut [u8]> {
        if self.tx_buffer.is_empty() {
            None
        } else {
            let len = length.min(self.tx_buffer.len());
            Some(&mut self.tx_buffer[..len])
        }
    }

    /// Clears all DMA error flags
    pub fn clear_errors(&mut self) {
        if let Some(dma_rx) = &mut self.dma_rx {
            dma_rx.clear_transfer_error();
        }
        if let Some(dma_tx) = &mut self.dma_tx {
            dma_tx.clear_transfer_error();
        }
    }

    /// Clears DMA TX complete flag
    pub fn clear_dma_tx_complete_flag(&mut self) {
        if let Some(dma_tx) = &mut self.dma_tx {
            dma_tx.clear_flags(DmaFlag::FifoError | DmaFlag::TransferComplete);
        }
    }

    /// Clears DMA RX complete flag
    pub fn clear_dma_rx_complete_flag(&mut self) {
        if let Some(dma_rx) = &mut self.dma_rx {
            dma_rx.clear_flags(DmaFlag::FifoError | DmaFlag::TransferComplete);
        }
    }

    /// Checks if USART RX buffer is not empty
    pub fn is_rx_not_empty(&self) -> bool {
        let usart = unsafe { &*USART6::ptr() };
        usart.sr().read().rxne().bit_is_set()
    }

    /// Checks if USART TX buffer is empty
    pub fn is_tx_empty(&self) -> bool {
        let usart = unsafe { &*USART6::ptr() };
        usart.sr().read().txe().bit_is_set()
    }

    /// Checks if transmission is complete
    pub fn is_transmission_complete(&self) -> bool {
        let usart = unsafe { &*USART6::ptr() };
        usart.sr().read().tc().bit_is_set()
    }

    /// Clears specified USART flags using proper clear sequences
    ///
    /// # Parameters
    /// - `flags`: Combination of UsartFlag bits to clear
    pub fn clear_usart_flags(&self, flags: UsartFlag) {
        let usart = unsafe { &*USART6::ptr() };
        let sr = usart.sr().read();

        if flags.contains(UsartFlag::RXNE) && sr.rxne().bit_is_set() {
            let _ = usart.dr().read().bits();
        }

        if flags.contains(UsartFlag::TXE) && sr.txe().bit_is_set() {
            usart.dr().write(|w| unsafe { w.bits(0) });
        }

        if flags.contains(UsartFlag::TC) && sr.tc().bit_is_set() {
            usart.dr().write(|w| unsafe { w.bits(0) });
        }

        #[cfg(feature = "debug")]
        defmt::trace!("Cleared USART flags: {:?}", flags);
    }

    /// Checks DMA RX idle state
    ///
    /// # Errors
    /// Returns `UsartError::NotInitialized` if DMA RX not configured
    pub fn is_dma_rx_is_idle(&self) -> Result<bool, UsartError> {
        self.dma_rx
            .as_ref()
            .ok_or(UsartError::NotInitialized)
            .map(|dma| dma.is_idle())
    }

    /// Stops ongoing transfers and cleans up resources
    pub fn stop_transfer(&mut self) {
        self.clear_errors();
        while let Ok(true) = self.is_dma_tx_complete() {
            cortex_m::asm::nop();
        }
    }

    /// Gets available data size in TX buffer
    pub fn available_data(&mut self) -> usize {
        self.is_dma_tx_complete()
            .map(|complete| if complete { DMA_BUFFER_LEN } else { 0 })
            .unwrap_or(0)
    }

    /// Gets current number of transfers configured in DMA RX stream
    ///
    /// # Example
    /// ```rust
    /// let length = usart.get_dma_rx_length()?;
    /// defmt::info!("DMA RX transfers: {}", length);
    /// ```
    ///
    /// # Errors
    /// Returns `UsartError::NotInitialized` if DMA RX not configured
    pub fn get_dma_rx_length(&mut self) -> Result<usize, UsartError> {
        let dma = self.dma_rx.as_mut().ok_or(UsartError::NotInitialized)?;

        // SAFETY: Direct register access wrapped in HAL methods
        let transfers = unsafe { dma.stream().number_of_transfers() };

        #[cfg(feature = "debug")]
        defmt::trace!("DMA RX length: {}", transfers);

        Ok(transfers as usize)
    }
}

/// Automatic cleanup implementation
impl Drop for Usart6Controller {
    fn drop(&mut self) {
        self.clear_errors();
        #[cfg(feature = "debug")]
        defmt::info!("USART6 controller released");
    }
}
