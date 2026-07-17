//! Derive bus clocks from the carrier target clock tree.
//!
//! On silicon, SYSCLK is HSI (64 MHz) at reset and only reaches 480 MHz after
//! HSE + PLL1 bring-up. ArmOS and `stm32h745-carrier` QEMU both treat the
//! **post-PLL** point as the operating configuration:
//!
//! ```text
//! HSE 8 MHz → PLL1 → SYSCLK 480 MHz → HCLK (AHB) → PCLK1/2 (APB)
//! ```
//!
//! Prescalers are read from RCC when programmed; until then they are 1.

#![cfg_attr(feature = "mps2", allow(dead_code))]

use crate::board;

use super::pac;

pub fn pclk2_hz() -> u32 {
    let rcc = pac::rcc();
    let hclk = hclk_hz(rcc);
    let prescaler = apb2_prescaler(rcc.d2cfgr().read().d2ppre2().bits());
    hclk / prescaler
}

pub fn sysclk_hz() -> u32 {
    // Target full-speed operating point (matches QEMU SYSCLK and silicon after PLL).
    // When live SWS/PLL decoding is added, read CFGR/PLLCKSELR/… here instead.
    board::SYSCLK_FREQ_HZ
}

fn hclk_hz(rcc: &pac::device::rcc::RegisterBlock) -> u32 {
    let sysclk = sysclk_hz();
    let hpre = rcc.d1cfgr().read().hpre().bits();
    sysclk / hpre_divisor(hpre)
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
