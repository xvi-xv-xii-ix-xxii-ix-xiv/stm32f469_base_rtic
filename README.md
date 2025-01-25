# Universal Embedded Development Template for RTIC-Based Projects (Rust)

[![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue)](https://opensource.org/licenses/MIT)

**A Modular Foundation for STM32 and Compatible Microcontrollers**  
This repository serves as a **starting point** for embedded projects - not a final product, but a *curated collection of reusable patterns* that can be:
- 🧩 **Used selectively** - integrate only required components into your project
- 🎛 **Adapted across MCUs** - supports STM32F4/F7/H7, GD32, Nordic, and other Cortex-M chips
- 🏗 **Extended like LEGO** - combine modules to accelerate development

## Why Choose This Template?

### For STM32F469 Discovery (Default Configuration)
Provides ready-to-run foundations for STM32F469NI-Discovery with:
- Pre-optimized clock tree configuration (180 MHz PLL)
- Pre-wired peripherals (USB OTG FS, USART6 with DMA, GPIO mappings)
- Reference implementations for board-specific features (LCD, touch sensors)

### For Other Microcontrollers
1. **Easy Adaptation** via:
  - Dependency swaps in `Cargo.toml`

## Key Features

### Core Capabilities
- 🕒 **RTIC Framework**: Preconfigured real-time task scheduling with:
  - Priority-based execution (1-5 levels)
  - Safe resource sharing via mutexes
  - Async/await support for non-blocking operations

- 📡 **Communication Protocols**:
  - **DMA-Driven USART6**:
    - 115200 baud rate (configurable)
    - Hardware flow control (RTS/CTS)
    - Circular buffer management (256-byte capacity)
  - **USB 2.0 OTG FS**:
    - CDC-ACM virtual COM port
    - Bulk data transfer support
    - Plug-and-play enumeration

- 🚦 **Visual Status System**:
  - Blue LED (PK3): Operational status patterns
    - Steady blink: Normal operation
    - Rapid blink: Data transmission
    - Off: Error state
  - Red LED (PD5): Error code visualization
    - Morse-like coding for error identification
    - Persistent error logging

### Advanced Functionality
- 🔋 **Power Management**:
  - Automatic entry into STOP mode during idle
  - < 1µA sleep current (peripheral-dependent)
  - Interrupt-driven wakeup system

- 🛡️ **Error Handling**:
  - Hierarchical error domains (USB, DMA, USART)
  - Persistent error queue (8-entry FIFO)
  - Error code-to-description mapping
  - Cross-domain error conversion

- 📊 **Debug Infrastructure**:
  - Conditional debug output via RTT
  - Panic handler with stack trace
  - Performance metrics:
    - ISR latency measurements
    - CPU load monitoring
    - Buffer utilization stats

## Hardware Integration

### Supported Peripherals
| Peripheral  | Features                          | GPIO Mapping          |
|-------------|-----------------------------------|-----------------------|
| USART6      | DMA TX/RX, Hardware Flow Control  | TX: PG14, RX: PG9     |
| USB OTG FS  | Device Mode, VBUS Sensing         | DP: PA11, DN: PA12    |
| GPIO        | LED Control, User Input           | PK3 (Blue), PD5 (Red) |
| SYSTICK     | System Timer                      | Core-integrated       |
| DMA2        | Stream Management                 | Channel 4/5           |

## Development Workflow

### Getting Started
1. Clone template repository:
   ```bash
   git clone https://github.com/xvi-xv-xii-ix-xxii-ix-xiv/stm32f469_base_rtic.git

## Safety-Critical Design

### Protection Mechanisms

- **Memory Safety**:
  - Guaranteed buffer boundaries for DMA
  - Stack canary protection
  - Watchdog timer integration
- **Fault Recovery**:
  - Automatic retry for failed transfers (3 attempts)
  - Graceful degradation on critical errors
  - Hardware watchdog kick system

License

This project is licensed under the MIT License. See the LICENSE file for details.

Author

Developed by XVI.XV.XII.IX.XXII.IX.XIV
