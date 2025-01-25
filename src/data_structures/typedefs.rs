use stm32f4xx_hal::{
    dma::{MemoryToPeripheral, PeripheralToMemory, Stream1, Stream6, Transfer},
    pac::{self, DMA2},
    serial::{Rx, Tx},
};

/// Type for transmitting data via DMA (TX).
///
/// This type represents a DMA transfer from memory to the USART6 TX pin,
/// using the `Stream6` of `DMA2`. It handles the data transfer from a
/// static mutable byte slice to the USART TX peripheral.
pub type DmaTxTransfer =
    Transfer<Stream6<DMA2>, 5, Tx<pac::USART6>, MemoryToPeripheral, &'static mut [u8]>;

/// Type for receiving data via DMA (RX).
///
/// This type represents a DMA transfer from the USART6 RX pin to memory,
/// using the `Stream1` of `DMA2`. It handles the data transfer from the
/// USART RX peripheral to a static mutable byte slice in memory.
pub type DmaRxTransfer =
    Transfer<Stream1<DMA2>, 5, Rx<pac::USART6>, PeripheralToMemory, &'static mut [u8]>;
