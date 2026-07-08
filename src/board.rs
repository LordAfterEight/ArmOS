//! Carrier-board configuration — update when the schematic is finalized.

#![cfg_attr(feature = "qemu", allow(dead_code))]

/// USART1 TX pin (default: PA9, AF7). Change for your carrier board.
pub const USART1_TX: GpioPin = GpioPin {
    port: GpioPort::A,
    pin: 9,
    alternate: 7,
};

/// USART1 RX pin (default: PA10, AF7). Optional for boot proof (TX only).
pub const USART1_RX: GpioPin = GpioPin {
    port: GpioPort::A,
    pin: 10,
    alternate: 7,
};

pub const UART_BAUD: u32 = 115_200;

/// External HSE frequency in Hz, when populated on the carrier board.
pub const HSE_FREQ_HZ: Option<u32> = None;

#[derive(Clone, Copy)]
pub enum GpioPort {
    A,
}

pub struct GpioPin {
    pub port: GpioPort,
    pub pin: u8,
    pub alternate: u8,
}