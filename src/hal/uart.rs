#![cfg_attr(feature = "qemu", allow(dead_code))]

use core::fmt::{self, Write};

use crate::board::{self, GpioPin};

use super::clock;
use super::regs::{
    gpio, modify_reg, rcc, usart, write_reg, GpioId, RCC_BASE, USART1_BASE,
};

#[cfg(feature = "qemu")]
use super::semihost;

struct UartWriter;

impl Write for UartWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_str(s);
        Ok(())
    }
}

pub fn init() {
    #[cfg(feature = "qemu")]
    {
        return;
    }

    #[cfg(not(feature = "qemu"))]
    init_hw();
}

#[cfg(not(feature = "qemu"))]
fn init_hw() {
    enable_gpio_clock(board::USART1_TX.port);
    enable_usart1_clock();
    configure_pin(&board::USART1_TX);
    configure_pin(&board::USART1_RX);
    configure_usart1();
}

fn enable_gpio_clock(port: GpioId) {
    if port.base == crate::hal::regs::GPIOA_BASE {
        modify_reg(RCC_BASE, rcc::AHB4ENR, rcc::GPIOAEN, rcc::GPIOAEN);
    }
}

fn enable_usart1_clock() {
    modify_reg(RCC_BASE, rcc::APB2ENR, rcc::USART1EN, rcc::USART1EN);
}

fn configure_pin(pin: &GpioPin) {
    let n = pin.pin as u32;
    let moder_shift = n * 2;
    modify_reg(pin.port.base, gpio::MODER, 0x3 << moder_shift, 0x2 << moder_shift);

    let afr_idx = (n / 8) as usize;
    let afr_shift = (n % 8) * 4;
    modify_reg(
        pin.port.base,
        gpio::AFR[afr_idx],
        0xF << afr_shift,
        (pin.alternate as u32) << afr_shift,
    );
}

fn configure_usart1() {
    let pclk = clock::pclk2_hz();
    let brr = (pclk + (board::UART_BAUD / 2)) / board::UART_BAUD;
    write_reg(USART1_BASE, usart::BRR, brr);
    write_reg(USART1_BASE, usart::CR1, usart::UE | usart::TE);
}

pub fn write_str(s: &str) {
    #[cfg(feature = "qemu")]
    semihost::write_str(s);

    #[cfg(not(feature = "qemu"))]
    for &b in s.as_bytes() {
        write_byte(b);
    }
}

pub fn write_byte(b: u8) {
    #[cfg(feature = "qemu")]
    {
        let one = [b];
        semihost::write_str(core::str::from_utf8(&one).unwrap_or(""));
    }

    #[cfg(not(feature = "qemu"))]
    {
        while super::regs::read_reg(USART1_BASE, usart::ISR) & usart::TXE == 0 {}
        write_reg(USART1_BASE, usart::TDR, b as u32);
    }
}

pub fn write_fmt(args: fmt::Arguments<'_>) {
    let _ = UartWriter.write_fmt(args);
}