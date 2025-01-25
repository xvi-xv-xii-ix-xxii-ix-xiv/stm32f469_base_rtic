/// Macro for creating default DMA configuration with common settings
///
/// Generates a preconfigured `DmaConfig` with:
/// - Transfer Complete Interrupt enabled
/// - FIFO enabled with QuarterFull threshold
/// - High priority
/// - Memory increment mode
#[macro_export]
macro_rules! dma_cfg {
    () => {
        stm32f4xx_hal::dma::config::DmaConfig::default()
            .transfer_complete_interrupt(true)
            .transfer_error_interrupt(false)
            .half_transfer_interrupt(false)
            .fifo_enable(true)
            .fifo_threshold(stm32f4xx_hal::dma::config::FifoThreshold::QuarterFull)
            .peripheral_burst(stm32f4xx_hal::dma::config::BurstMode::NoBurst)
            .peripheral_increment(false)
            .memory_burst(stm32f4xx_hal::dma::config::BurstMode::NoBurst)
            .memory_increment(true)
            .priority(stm32f4xx_hal::dma::config::Priority::High)
    };
}