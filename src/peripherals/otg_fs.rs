//! # USB OTG FS Controller Implementation
//!
//! This module provides USB device functionality using the OTG FS peripheral
//! on STM32F4 microcontrollers. Key features include:
//! - USB Serial Communication Device Class (CDC) implementation
//! - Dual buffer management for RX/TX operations
//! - Atomic state tracking for USB initialization
//! - Error handling for USB communication faults
//!
//! ## Hardware Configuration
//! - Uses PA11 (DM) and PA12 (DP) pins in alternate function mode 10
//! - Requires OTG FS global, device, and power/clock registers
//! - Buffer sizes configured in `config` module

use core::sync::atomic::{AtomicBool, Ordering};
use stm32f4xx_hal::pac::{OTG_FS_DEVICE, OTG_FS_GLOBAL, OTG_FS_PWRCLK};
use stm32f4xx_hal::{
    gpio::{
        gpioa::{PA11, PA12},
        Alternate,
    },
    otg_fs::{UsbBusType, USB},
};
use usb_device::{
    class_prelude::UsbBusAllocator,
    device::{StringDescriptors, UsbDeviceBuilder, UsbVidPid},
    prelude::*,
};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use crate::config::{DATA_PACKET_SIZE, OTG_FS_BUFFER_LEN};
use crate::errors::errors::UsbError;
use crate::peripherals::rcc::RccConfig;

/// Shared USB bus allocator (singleton pattern)
static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;

/// Atomic state tracking for USB initialization
static USB_BUS_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// USB OTG FS controller with serial CDC support
pub struct OtgFsController<'a> {
    pub(crate) usb_device: Option<UsbDevice<'a, UsbBusType>>,
    pub(crate) serial: Option<SerialPort<'a, UsbBusType>>,
    rx_buffer: [u8; DATA_PACKET_SIZE],
    tx_buffer: [u8; DATA_PACKET_SIZE],
}

impl<'a> OtgFsController<'a> {
    /// Initializes USB OTG FS controller
    ///
    /// # Arguments
    /// * `otg_fs_global` - OTG FS global registers
    /// * `otg_fs_device` - OTG FS device registers
    /// * `otg_fs_pwrclk` - OTG FS power/clock registers
    /// * `dm_pin` - USB D- pin (PA11)
    /// * `dp_pin` - USB D+ pin (PA12)
    /// * `clocks` - Clock configuration
    ///
    /// # Errors
    /// Returns `UsbError::NotInitialized` if:
    /// - USB peripheral initialization fails
    /// - Buffer allocation fails
    ///
    /// # Safety
    /// - Must be called only once during system initialization
    pub fn new(
        otg_fs_global: OTG_FS_GLOBAL,
        otg_fs_device: OTG_FS_DEVICE,
        otg_fs_pwrclk: OTG_FS_PWRCLK,
        dm_pin: PA11<Alternate<10>>,
        dp_pin: PA12<Alternate<10>>,
        clocks: &'a RccConfig,
    ) -> Result<Self, UsbError> {
        if USB_BUS_INITIALIZED.load(Ordering::SeqCst) {
            return Err(UsbError::NotInitialized);
        }

        // Initialize USB peripheral
        let usb = USB::new(
            (otg_fs_global, otg_fs_device, otg_fs_pwrclk),
            (dm_pin, dp_pin),
            &clocks.clocks,
        );

        // Allocate USB endpoint memory
        let usb_ep_memory: &'static mut [u32; OTG_FS_BUFFER_LEN] =
            cortex_m::singleton!(: [u32; OTG_FS_BUFFER_LEN] = [0; OTG_FS_BUFFER_LEN])
                .ok_or(UsbError::NotInitialized)?;

        let (usb_device, serial) = unsafe {
            // Инициализация USB шины
            USB_BUS = Some(UsbBusType::new(usb, usb_ep_memory));

            #[allow(static_mut_refs)]
            let bus_ref = USB_BUS.as_ref().unwrap();

            let serial = SerialPort::new(bus_ref);
            let usb_device = UsbDeviceBuilder::new(bus_ref, UsbVidPid(0x16c0, 0x27dd))
                .device_class(USB_CLASS_CDC)
                .strings(&[StringDescriptors::default()
                    .manufacturer("xvi.xv.xii.ix.xxii.ix.xiv")
                    .product("USB-Serial Bridge")
                    .serial_number("007")])
                .unwrap()
                .build();

            (Some(usb_device), Some(serial))
        };

        USB_BUS_INITIALIZED.store(true, Ordering::SeqCst);

        Ok(Self {
            usb_device,
            serial,
            rx_buffer: [0; DATA_PACKET_SIZE],
            tx_buffer: [0; DATA_PACKET_SIZE],
        })
    }

    /// Reads data from USB interface
    ///
    /// # Returns
    /// - `Ok(Some((data, len)))` on successful read with data slice and length
    /// - `Ok(None)` when no data available
    /// - `Err(UsbError)` on communication failure
    pub fn read(&mut self) -> Result<Option<(&[u8], usize)>, UsbError> {
        let serial = self.serial.as_mut().ok_or(UsbError::NotInitialized)?;

        match serial.read(&mut self.rx_buffer) {
            Ok(count) if count > 0 => Ok(Some((&self.rx_buffer[..count], count))),
            Ok(_) => Ok(None),
            Err(usb_device::UsbError::WouldBlock) => {
                #[cfg(feature = "debug")]
                defmt::trace!("USB read would block");
                Ok(None)
            }
            Err(_) => {
                #[cfg(feature = "debug")]
                defmt::error!("USB read error");
                Err(UsbError::ReadError)
            }
        }
    }

    /// Writes data to USB interface
    ///
    /// # Arguments
    /// * `data` - Slice of data to transmit
    ///
    /// # Errors
    /// Returns `UsbError` if:
    /// - Data exceeds buffer size
    /// - Write operation fails
    pub fn write(&mut self, data: &[u8]) -> Result<usize, UsbError> {
        let serial = self.serial.as_mut().ok_or(UsbError::NotInitialized)?;

        if data.len() > DATA_PACKET_SIZE {
            return Err(UsbError::BufferOverflow);
        }

        self.tx_buffer[..data.len()].copy_from_slice(data);

        serial
            .write(&self.tx_buffer[..data.len()])
            .map_err(|_| UsbError::WriteError)
    }

    /// Polls USB device state and handles events
    ///
    /// # Returns
    /// `true` if device is active and polled successfully
    pub fn poll(&mut self) -> bool {
        if let Some(usb_dev) = &mut self.usb_device {
            if let Some(serial) = &mut self.serial {
                usb_dev.poll(&mut [serial]);

                #[cfg(feature = "debug")]
                match usb_dev.state() {
                    UsbDeviceState::Configured => defmt::debug!("USB configured"),
                    UsbDeviceState::Addressed => defmt::trace!("USB addressed"),
                    UsbDeviceState::Default => defmt::trace!("USB default state"),
                    UsbDeviceState::Suspend => defmt::warn!("USB suspended"),
                }

                return true;
            }
        }
        false
    }

    /// Checks if USB device is in configured state
    pub fn is_configured(&self) -> bool {
        self.usb_device
            .as_ref()
            .map_or(false, |dev| dev.state() == UsbDeviceState::Configured)
    }

    /// Activates USB controller and enables interrupts
    ///
    /// # Errors
    /// Returns `UsbError` if initialization check fails
    pub fn start(&mut self) -> Result<(), UsbError> {
        let usb_dev = self.usb_device.as_mut().ok_or(UsbError::NotInitialized)?;
        let _serial = self.serial.as_mut().ok_or(UsbError::NotInitialized)?;

        // SAFETY: Single interrupt unmask operation
        unsafe {
            cortex_m::peripheral::NVIC::unmask(stm32f4xx_hal::pac::Interrupt::OTG_FS);
        }

        if !usb_dev.poll(&mut []) {
            return Err(UsbError::InitError);
        }

        Ok(())
    }

    /// Returns mutable reference to RX buffer
    pub fn get_rx_buffer(&mut self) -> &mut [u8] {
        &mut self.rx_buffer
    }

    /// Returns mutable reference to TX buffer
    pub fn get_tx_buffer(&mut self) -> &mut [u8] {
        &mut self.tx_buffer
    }
}

/// Cleanup implementation
impl<'a> Drop for OtgFsController<'a> {
    fn drop(&mut self) {
        // SAFETY: Exclusive access during destruction
        unsafe {
            USB_BUS = None;
        }
        USB_BUS_INITIALIZED.store(false, Ordering::SeqCst);

        #[cfg(feature = "debug")]
        defmt::info!("USB controller released");

        self.rx_buffer.fill(0);
        self.tx_buffer.fill(0);
    }
}
