//! Derive USART1 kernel clock (PCLK2) from RCC at runtime.

#![cfg_attr(feature = "qemu", allow(dead_code))]

use super::pac;

const HSI_HZ: u32 = 64_000_000;

pub fn pclk2_hz() -> u32 {
    let rcc = pac::rcc();
    let hclk = hclk_hz(rcc);
    let prescaler = apb2_prescaler(rcc.d2cfgr().read().d2ppre2().bits());
    hclk / prescaler
}

fn hclk_hz(rcc: &pac::device::rcc::RegisterBlock) -> u32 {
    let sysclk = sysclk_hz();
    let hpre = rcc.d1cfgr().read().hpre().bits();
    sysclk / hpre_divisor(hpre)
}

fn sysclk_hz() -> u32 {
    // Reset default: HSISYS selected (CFGR SWS == 0).
    HSI_HZ
}

fn hpre_divisor(code: u8) -> u32 {
    match code {
        0x8 => 2,
        0x9 => 4,
        0xA => 8,
        0xB => 16,
        0xC => 64,
        0xD => 128,
        0xE => 256,
        0xF => 512,
        _ => 1,
    }
}

fn apb2_prescaler(code: u8) -> u32 {
    match code {
        0x4 => 2,
        0x5 => 4,
        0x6 => 8,
        0x7 => 16,
        _ => 1,
    }
}