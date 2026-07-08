//! Carrier-board configuration — update when the schematic is finalized.

#![cfg_attr(feature = "qemu", allow(dead_code))]

use crate::hal::regs::{GpioId, GPIOA};

/// USART1 TX pin (default: PA9, AF7). Change for your carrier board.
pub const USART1_TX: GpioPin = GpioPin {
    port: GPIOA,
    pin: 9,
    alternate: 7,
};

/// USART1 RX pin (default: PA10, AF7). Optional for boot proof (TX only).
pub const USART1_RX: GpioPin = GpioPin {
    port: GPIOA,
    pin: 10,
    alternate: 7,
};

pub const UART_BAUD: u32 = 115_200;

/// External HSE frequency in Hz, when populated on the carrier board.
pub const HSE_FREQ_HZ: Option<u32> = None;

pub struct GpioPin {
    pub port: GpioId,
    pub pin: u8,
    pub alternate: u8,
}