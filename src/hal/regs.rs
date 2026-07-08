//! STM32H745 register addresses (RM0433).

#![cfg_attr(feature = "qemu", allow(dead_code))]

pub const RCC_BASE: usize = 0x5802_4400;
pub const GPIOA_BASE: usize = 0x5802_0000;
pub const USART1_BASE: usize = 0x4001_1000;

pub struct GpioId {
    pub base: usize,
}

pub const GPIOA: GpioId = GpioId { base: GPIOA_BASE };

pub mod rcc {
    pub const CFGR: usize = 0x10;
    pub const D2CFGR: usize = 0x8C;
    pub const AHB4ENR: usize = 0xE0;
    pub const APB2ENR: usize = 0xF0;

    pub const GPIOAEN: u32 = 1 << 0;
    pub const USART1EN: u32 = 1 << 4;
}

pub mod gpio {
    pub const MODER: usize = 0x00;
    pub const AFR: [usize; 2] = [0x20, 0x24];
}

pub mod usart {
    pub const CR1: usize = 0x00;
    pub const BRR: usize = 0x0C;
    pub const ISR: usize = 0x1C;
    pub const TDR: usize = 0x28;

    pub const UE: u32 = 1 << 0;
    pub const TE: u32 = 1 << 3;
    pub const TXE: u32 = 1 << 7;
}

#[inline(always)]
pub fn read_reg(base: usize, offset: usize) -> u32 {
    unsafe { core::ptr::read_volatile((base + offset) as *const u32) }
}

#[inline(always)]
pub fn write_reg(base: usize, offset: usize, value: u32) {
    unsafe { core::ptr::write_volatile((base + offset) as *mut u32, value) }
}

#[inline(always)]
pub fn modify_reg(base: usize, offset: usize, mask: u32, value: u32) {
    let old = read_reg(base, offset);
    write_reg(base, offset, (old & !mask) | (value & mask));
}