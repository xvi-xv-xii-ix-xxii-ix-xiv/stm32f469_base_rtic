use crate::{define_peripheral_error_enum, impl_error_conversion};
// ========================
// Ring Buffer Error Domain
// ========================

define_peripheral_error_enum!(
    RingBufferError,
    BufferOverflow => "Ring buffer overflow occurred",
    InsufficientSpace => "Insufficient space in ring buffer",
    BufferEmpty => "Ring buffer is empty"
);

// =================
// LED Error Domain
// =================

define_peripheral_error_enum!(
    LedError,
    SetStateError => "Failed to set LED state",
    ReadStateError => "Failed to read LED state"
);

// ==================
// USART Error Domain
// ==================

define_peripheral_error_enum!(
    UsartError,
    DmaError => "DMA error occurred in USART",
    TransferError => "Transfer error in USART",
    Timeout => "USART operation timed out",
    NotInitialized => "USART not initialized",
    BufferOverflow => "USART buffer overflow",
    FlagNotSet => "USART flag not set",
);

// ================
// USB Error Domain
// ================

define_peripheral_error_enum!(
    UsbError,
    NotInitialized => "USB device is not initialized",
    ReadError => "Failed to read from USB",
    WriteError => "Failed to write to USB",
    BufferOverflow => "USB buffer overflow",
    InitError => "Failed to initialize USB",
    PollError => "Failed to poll USB"
);

// =================
// DMA Error Domain
// =================

define_peripheral_error_enum!(
    DmaError,
    InitError => "Failed to initialize DMA",
    TransferError => "DMA transfer error",
    RetryLimitExceeded => "DMA retry limit exceeded",
    BufferOverflow => "DMA buffer overflow",
    BufferUnderflow => "DMA buffer underflow",
    WriteError => "Failed to write using DMA",
    ReadError => "Failed to read using DMA"
);

// ======================
// Device Error Domain
// ======================

define_peripheral_error_enum!(
    DeviceError,
    UsbError => "USB device error occurred",
    DmaError => "DMA error occurred",
    BufferOverflow => "Device buffer overflow",
    Timeout => "Operation timed out",
    LedError => "LED error occurred"
);

// ========================
// Initialization Errors
// ========================

define_peripheral_error_enum!(
    InitError,
    UsartError => "USART initialization error",
    UsbError => "USB initialization error",
    RccError => "RCC initialization error",
    LutError => "LUT initialization error"
);

// ==============================
// Error Conversion Implementations
// ==============================

impl_error_conversion!(UsbError, DeviceError, { UsbError });

impl_error_conversion!(DmaError, DeviceError, { DmaError });

impl_error_conversion!(UsartError, DeviceError, { DmaError });

impl_error_conversion!(LedError, DeviceError, { LedError });

impl_error_conversion!(RingBufferError, DeviceError, { BufferOverflow });