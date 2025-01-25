/// Length of the DMA buffer (Direct Memory Access buffer size).
/// This constant defines the number of bytes that the DMA buffer can hold.
pub const DMA_BUFFER_LEN: usize = 128;

/// Length of the USB OTG FS buffer.
/// This constant sets the maximum size for the USB OTG FS buffer in bytes.
pub const OTG_FS_BUFFER_LEN: usize = 1024;

/// Length of the ring buffer.
/// This constant specifies the number of elements the ring buffer can store.
pub const RING_BUFFER_LEN: usize = 512;

/// Size of each data packet.
/// This constant defines the size of each data packet, typically used in data communication.
/// It is set to 256 bytes, which is often used in custom communication protocols.
pub const DATA_PACKET_SIZE: usize = 128;

/// USART6 baud rate.
/// Sets the data transmission speed for USART6 communication, typically in bits per second.
/// The baud rate is set to 115200, which is a common rate for serial communication.
pub const USART6_BAUD_RATE: u32 = 115200;

/// High-Speed External clock frequency (HSE).
/// Represents the external oscillator frequency connected to the microcontroller.
/// This value is typically set to 8 MHz for STM32F4 series microcontrollers.
pub const HSE: u32 = 8_000_000;

/// System clock frequency (SYSCLK).
/// This constant defines the main system clock frequency. It is set to 180 MHz,
/// derived from the PLL configuration of the STM32F4 microcontroller.
pub const SYSCLK: u32 = 180_000_000;

/// Peripheral clock 1 frequency (PCLK1).
/// This constant defines the clock frequency used for peripherals connected to the APB1 bus.
/// It is set to 45 MHz, based on the system clock configuration and dividers.
pub const PCLK1: u32 = 45_000_000;

/// Peripheral clock 2 frequency (PCLK2).
/// This constant defines the clock frequency used for peripherals connected to the APB2 bus.
/// It is set to 90 MHz and is derived from SYSCLK with the appropriate dividers.
pub const PCLK2: u32 = 90_000_000;

/// Maximum Morse code sequence length.
/// Defines the maximum allowed length for a Morse code sequence, measured in characters or signals.
/// This is typically used for buffer allocation and validation purposes.
pub const MAX_MORSE_LENGTH: usize = 100;
