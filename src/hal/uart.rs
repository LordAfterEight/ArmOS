#![cfg_attr(feature = "qemu", allow(dead_code))]

use core::fmt::{self, Write};

#[cfg(not(feature = "qemu"))]
use crate::board;

#[cfg(not(feature = "qemu"))]
use super::clock;
#[cfg(not(feature = "qemu"))]
use super::gpio;
#[cfg(not(feature = "qemu"))]
use super::pac::{self, device};

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
    // Single-core bring-up: steal after reset (no allocator/concurrency yet).
    let dp = unsafe { device::Peripherals::steal() };

    dp.RCC.ahb4enr().modify(|_, w| w.gpioaen().set_bit());
    dp.RCC.apb2enr().modify(|_, w| w.usart1en().set_bit());

    gpio::configure_af_pin(&dp.GPIOA, &board::USART1_TX);
    gpio::configure_af_pin(&dp.GPIOA, &board::USART1_RX);

    let pclk = clock::pclk2_hz();
    let brr = (pclk + (board::UART_BAUD / 2)) / board::UART_BAUD;

    dp.USART1.brr().write(|w| unsafe { w.brr().bits(brr as u16) });
    dp.USART1.cr1().write(|w| w.ue().set_bit().te().set_bit());
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
        let usart1 = pac::usart1();
        while usart1.isr().read().txe().bit_is_clear() {}
        usart1.tdr().write(|w| unsafe { w.tdr().bits(b as u16) });
    }
}

pub fn write_fmt(args: fmt::Arguments<'_>) {
    let _ = UartWriter.write_fmt(args);
}