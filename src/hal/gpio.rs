//! GPIO abstraction — every pad is a [`Pin`]; hardwired pads need a feature.
//!
//! - `Pin<false>` — free / expansion / status LEDs → `into_output` / `into_input` always
//! - `Pin<true>` — FMC, QSPI, SWD, … → those methods only with
//!   **`--features gpio-allow-hardwired`** (otherwise: **compile error**, method missing)
//!
//! ```ignore
//! use ArmOS::board::gpio::{PE2, PE6, PF0};
//! use ArmOS::hal::gpio::{self, led};
//!
//! gpio::init_clocks();
//!
//! let mut free = PE6.into_output();     // always OK
//! free.set_high();
//!
//! led::set(led::RUN, true);             // PE2 via LED helper
//! let mut run = PE2.into_output();      // also OK (LED is not restricted)
//!
//! // let x = PF0.into_output();         // ERROR unless gpio-allow-hardwired
//! ```

use core::marker::PhantomData;

use crate::hal::pac::{self, device};

// ── Ports ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Port {
    A = 0,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
}

impl Port {
    #[inline]
    pub const fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::A),
            1 => Some(Self::B),
            2 => Some(Self::C),
            3 => Some(Self::D),
            4 => Some(Self::E),
            5 => Some(Self::F),
            6 => Some(Self::G),
            7 => Some(Self::H),
            8 => Some(Self::I),
            9 => Some(Self::J),
            10 => Some(Self::K),
            _ => None,
        }
    }

    #[inline]
    fn ahb4_bit(self) -> u32 {
        1u32 << (self as u8)
    }
}

// ── Pin<const RESTRICTED> ─────────────────────────────────────────────────

/// Physical pad. `RESTRICTED = true` means board hardwired function (FMC/QSPI/…).
///
/// Methods [`into_output`](Pin::into_output) / [`into_input`](Pin::into_input)
/// exist for `Pin<false>` always, and for `Pin<true>` only with feature
/// `gpio-allow-hardwired`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pin<const RESTRICTED: bool> {
    pub port: Port,
    pub num: u8,
}

impl<const RESTRICTED: bool> Pin<RESTRICTED> {
    pub const fn new(port: Port, num: u8) -> Self {
        Self { port, num }
    }

    /// Configure as push-pull output without the restricted-pin gate
    /// (board bring-up, LED helper, …).
    pub fn into_output_unchecked(self) -> Output {
        enable_port(self.port);
        configure_output(self.port, self.num);
        Output {
            port: self.port,
            num: self.num,
        }
    }

    pub fn into_input_unchecked(self) -> Input {
        enable_port(self.port);
        configure_input(self.port, self.num);
        Input {
            port: self.port,
            num: self.num,
        }
    }
}

impl Pin<false> {
    /// Claim as push-pull output.
    pub fn into_output(self) -> Output {
        self.into_output_unchecked()
    }

    /// Claim as floating input.
    pub fn into_input(self) -> Input {
        self.into_input_unchecked()
    }

    /// Claim as input with pull-up.
    pub fn into_input_pull_up(self) -> Input {
        enable_port(self.port);
        configure_input(self.port, self.num);
        set_pupdr(self.port, self.num, 0b01);
        Input {
            port: self.port,
            num: self.num,
        }
    }
}

#[cfg(feature = "gpio-allow-hardwired")]
impl Pin<true> {
    /// Claim a **hardwired** pad as GPIO (you override board function).
    pub fn into_output(self) -> Output {
        self.into_output_unchecked()
    }

    pub fn into_input(self) -> Input {
        self.into_input_unchecked()
    }

    pub fn into_input_pull_up(self) -> Input {
        enable_port(self.port);
        configure_input(self.port, self.num);
        set_pupdr(self.port, self.num, 0b01);
        Input {
            port: self.port,
            num: self.num,
        }
    }
}

// ── Handles ───────────────────────────────────────────────────────────────

pub struct Output {
    port: Port,
    num: u8,
}

impl Output {
    pub fn set_high(&mut self) {
        bsrr_set(self.port, self.num, true);
    }

    pub fn set_low(&mut self) {
        bsrr_set(self.port, self.num, false);
    }

    pub fn set(&mut self, high: bool) {
        bsrr_set(self.port, self.num, high);
    }

    pub fn toggle(&mut self) {
        let high = !is_odr_high(self.port, self.num);
        bsrr_set(self.port, self.num, high);
    }

    pub fn is_set_high(&self) -> bool {
        is_odr_high(self.port, self.num)
    }
}

pub struct Input {
    port: Port,
    num: u8,
}

impl Input {
    pub fn is_high(&self) -> bool {
        is_idr_high(self.port, self.num)
    }

    pub fn is_low(&self) -> bool {
        !self.is_high()
    }
}

// ── Status LEDs ───────────────────────────────────────────────────────────

pub mod led {
    use super::*;
    use crate::board::gpio::{PE2, PE3, PE4, PE5};

    pub const RUN: Pin<false> = PE2;
    pub const DBG: Pin<false> = PE3;
    pub const ERR: Pin<false> = PE4;
    pub const PNC: Pin<false> = PE5;

    pub fn init() {
        for p in [RUN, DBG, ERR, PNC] {
            let mut o = p.into_output();
            o.set_low();
        }
    }

    pub fn set(pin: Pin<false>, on: bool) {
        let mut o = pin.into_output();
        o.set(on);
    }
}

// ── Clocks ────────────────────────────────────────────────────────────────

pub fn init_clocks() {
    let rcc = pac::rcc();
    rcc.ahb4enr()
        .modify(|r, w| unsafe { w.bits(r.bits() | 0x7FF) });
    let _ = rcc.ahb4enr().read().bits();
}

fn enable_port(port: Port) {
    let rcc = pac::rcc();
    let bit = port.ahb4_bit();
    rcc.ahb4enr()
        .modify(|r, w| unsafe { w.bits(r.bits() | bit) });
    let _ = rcc.ahb4enr().read().bits();
}

// ── MMIO ──────────────────────────────────────────────────────────────────

type GpioRegs = device::gpioa::RegisterBlock;

fn regs(port: Port) -> &'static GpioRegs {
    let addr = match port {
        Port::A => 0x5802_0000usize,
        Port::B => 0x5802_0400,
        Port::C => 0x5802_0800,
        Port::D => 0x5802_0C00,
        Port::E => 0x5802_1000,
        Port::F => 0x5802_1400,
        Port::G => 0x5802_1800,
        Port::H => 0x5802_1C00,
        Port::I => 0x5802_2000,
        Port::J => 0x5802_2400,
        Port::K => 0x5802_2800,
    };
    unsafe { &*(addr as *const GpioRegs) }
}

fn configure_output(port: Port, num: u8) {
    set_moder(port, num, 0b01);
    set_otyper_pp(port, num);
    set_ospeed_high(port, num);
    set_pupdr(port, num, 0b00);
}

fn configure_input(port: Port, num: u8) {
    set_moder(port, num, 0b00);
    set_pupdr(port, num, 0b00);
}

fn set_moder(port: Port, num: u8, mode: u32) {
    let g = regs(port);
    let shift = (num as u32) * 2;
    g.moder().modify(|r, w| unsafe {
        w.bits((r.bits() & !(0b11 << shift)) | (mode << shift))
    });
}

fn set_otyper_pp(port: Port, num: u8) {
    let g = regs(port);
    g.otyper()
        .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << num)) });
}

fn set_ospeed_high(port: Port, num: u8) {
    let g = regs(port);
    let shift = (num as u32) * 2;
    g.ospeedr().modify(|r, w| unsafe {
        w.bits((r.bits() & !(0b11 << shift)) | (0b11 << shift))
    });
}

fn set_pupdr(port: Port, num: u8, pupd: u32) {
    let g = regs(port);
    let shift = (num as u32) * 2;
    g.pupdr().modify(|r, w| unsafe {
        w.bits((r.bits() & !(0b11 << shift)) | (pupd << shift))
    });
}

fn bsrr_set(port: Port, num: u8, high: bool) {
    let g = regs(port);
    let bit = if high {
        1u32 << num
    } else {
        1u32 << (num + 16)
    };
    g.bsrr().write(|w| unsafe { w.bits(bit) });
}

fn is_odr_high(port: Port, num: u8) -> bool {
    regs(port).odr().read().bits() & (1 << num) != 0
}

fn is_idr_high(port: Port, num: u8) -> bool {
    regs(port).idr().read().bits() & (1 << num) != 0
}

// ── Legacy AF (USART) ─────────────────────────────────────────────────────

pub fn configure_af_pin(gpio: &device::gpioa::RegisterBlock, pin: &crate::board::GpioPin) {
    let n = pin.pin;
    let shift = (n as u32) * 2;
    gpio.moder().modify(|r, w| unsafe {
        w.bits((r.bits() & !(0b11 << shift)) | (0b10 << shift))
    });
    let af = pin.alternate as u32;
    if n < 8 {
        let s = (n as u32) * 4;
        gpio.afrl().modify(|r, w| unsafe {
            w.bits((r.bits() & !(0xF << s)) | (af << s))
        });
    } else {
        let s = ((n - 8) as u32) * 4;
        gpio.afrh().modify(|r, w| unsafe {
            w.bits((r.bits() & !(0xF << s)) | (af << s))
        });
    }
    let _ = PhantomData::<()>;
}
