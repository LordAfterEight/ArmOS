//! Every MCU GPIO pad on the carrier, always named and always usable as a value.
//!
//! Hardwired pins (FMC, QSPI, SWD, LEDs, …) are still **constants** — they just
//! carry `Pin<true>`. Claiming them as general I/O requires Cargo feature
//! `gpio-allow-hardwired` (otherwise `into_output` / `into_input` do not exist
//! on that type → **compile error**).
//!
//! Source: KiCad netlist + QSPI NCS bodge (PG6).

use crate::hal::gpio::{Pin, Port};

// ── Roles (documentation / diagnostics only) ──────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinRole {
    /// Not hardwired to a board function — free to use.
    Free,
    /// On expansion header J2.
    Expansion,
    /// Status LED anode (PE2–PE5).
    Led,
    /// Used by FMC / QSPI / SWD / OSC / display / console / etc.
    Hardwired,
    /// Not bonded on this package (or not present).
    Absent,
}

pub const fn role(port: Port, num: u8) -> PinRole {
    if num > 15 {
        return PinRole::Absent;
    }
    match port {
        Port::A => role_a(num),
        Port::B => role_b(num),
        Port::C => role_c(num),
        Port::D => role_d(num),
        Port::E => role_e(num),
        Port::F => role_f(num),
        Port::G => role_g(num),
        Port::H => role_h(num),
        Port::I => role_i(num),
        Port::J => role_j(num),
        Port::K => role_k(num),
    }
}

/// `true` if the pad is hardwired or absent (needs `gpio-allow-hardwired` to claim).
pub const fn is_restricted(port: Port, num: u8) -> bool {
    match role(port, num) {
        PinRole::Free | PinRole::Expansion | PinRole::Led => false,
        // LEDs: claimable without the flag via into_output (Led is intentional I/O).
        // User asked all pins accessible; LEDs are meant to be driven.
        // Restricted = Hardwired | Absent only.
        PinRole::Hardwired | PinRole::Absent => true,
    }
}

/// LEDs are hardwired to diodes but **are** meant to be driven as GPIO.
/// Only FMC/QSPI/SWD/etc. are restricted.
pub const fn requires_allow_flag(port: Port, num: u8) -> bool {
    matches!(role(port, num), PinRole::Hardwired | PinRole::Absent)
}

const fn role_a(n: u8) -> PinRole {
    match n {
        13 => PinRole::Hardwired, // SWDIO
        14 => PinRole::Hardwired, // SWCLK
        9 | 10 => PinRole::Hardwired, // USART1 TX/RX (console)
        0..=8 | 11 | 12 | 15 => PinRole::Expansion,
        _ => PinRole::Absent,
    }
}

const fn role_b(n: u8) -> PinRole {
    match n {
        3 => PinRole::Hardwired, // SWO
        0..=2 | 4..=15 => PinRole::Expansion,
        _ => PinRole::Absent,
    }
}

const fn role_c(n: u8) -> PinRole {
    match n {
        0 => PinRole::Hardwired,  // FMC_WE
        2 => PinRole::Hardwired,  // FMC_CS / SDNE0
        6 => PinRole::Hardwired,  // FMC_NWAIT
        8 => PinRole::Hardwired,  // FMC_NCE
        1 | 3 | 4 | 5 | 7 | 9 | 10 | 11 | 12 | 13 | 14 | 15 => PinRole::Free,
        _ => PinRole::Absent,
    }
}

const fn role_d(n: u8) -> PinRole {
    match n {
        0 | 1 | 4 | 5 | 8 | 9 | 10 | 11 | 12 | 14 | 15 => PinRole::Hardwired,
        2 | 3 | 6 | 7 | 13 => PinRole::Free,
        _ => PinRole::Absent,
    }
}

const fn role_e(n: u8) -> PinRole {
    match n {
        0 | 1 => PinRole::Hardwired, // FMC NBL
        2 | 3 | 4 | 5 => PinRole::Led, // RUN DBG ERR PNC
        6 => PinRole::Free,
        7..=15 => PinRole::Hardwired, // FMC D4–D12
        _ => PinRole::Absent,
    }
}

const fn role_f(n: u8) -> PinRole {
    match n {
        0..=15 => PinRole::Hardwired, // FMC addr + QSPI
        _ => PinRole::Absent,
    }
}

const fn role_g(n: u8) -> PinRole {
    match n {
        0 | 1 | 2 | 3 | 4 | 5 | 8 | 15 => PinRole::Hardwired,
        6 => PinRole::Hardwired, // QSPI NCS (bodge)
        7 | 9 | 10 | 11 | 12 | 13 | 14 => PinRole::Free,
        _ => PinRole::Absent,
    }
}

const fn role_h(n: u8) -> PinRole {
    match n {
        0 => PinRole::Hardwired, // HSE OSC_IN
        7 => PinRole::Hardwired, // FMC_CKE
        1 => PinRole::Free,
        2..=6 | 8..=15 => PinRole::Hardwired, // display connector PH6
        _ => PinRole::Absent,
    }
}

const fn role_i(n: u8) -> PinRole {
    match n {
        11 => PinRole::Hardwired, // QSPI_RES
        0..=10 | 12..=15 => PinRole::Free,
        _ => PinRole::Absent,
    }
}

const fn role_j(n: u8) -> PinRole {
    match n {
        6..=11 => PinRole::Free,
        0..=5 | 12..=15 => PinRole::Absent,
        _ => PinRole::Absent,
    }
}

const fn role_k(n: u8) -> PinRole {
    match n {
        0..=2 => PinRole::Free,
        _ => PinRole::Absent,
    }
}

// ── All pads: Pin<false> free to claim, Pin<true> needs feature ───────────

macro_rules! pin_free {
    ($( $(#[$attr:meta])* $name:ident : $port:ident $num:literal ),* $(,)?) => {
        $(
            $(#[$attr])*
            pub const $name: Pin<false> = Pin::new(Port::$port, $num);
        )*
    };
}

macro_rules! pin_hw {
    ($( $(#[$attr:meta])* $name:ident : $port:ident $num:literal ),* $(,)?) => {
        $(
            $(#[$attr])*
            pub const $name: Pin<true> = Pin::new(Port::$port, $num);
        )*
    };
}

// Port A
pin_free! {
    /// Expansion J2.
    PA0: A 0,
    /// Expansion J2.
    PA1: A 1,
    /// Expansion J2.
    PA2: A 2,
    /// Expansion J2.
    PA3: A 3,
    /// Expansion J2.
    PA4: A 4,
    /// Expansion J2.
    PA5: A 5,
    /// Expansion J2.
    PA6: A 6,
    /// Expansion J2.
    PA7: A 7,
    /// Expansion J2.
    PA8: A 8,
    /// Expansion J2.
    PA11: A 11,
    /// Expansion J2.
    PA12: A 12,
    /// Expansion J2.
    PA15: A 15,
}
pin_hw! {
    /// **Hardwired:** USART1_TX (console).
    PA9: A 9,
    /// **Hardwired:** USART1_RX (console).
    PA10: A 10,
    /// **Hardwired:** SWDIO (debug).
    PA13: A 13,
    /// **Hardwired:** SWCLK (debug).
    PA14: A 14,
}

// Port B
pin_free! {
    /// Expansion J2.
    PB0: B 0,
    /// Expansion J2.
    PB1: B 1,
    /// Expansion J2.
    PB2: B 2,
    /// Expansion J2.
    PB4: B 4,
    /// Expansion J2.
    PB5: B 5,
    /// Expansion J2.
    PB6: B 6,
    /// Expansion J2.
    PB7: B 7,
    /// Expansion J2.
    PB8: B 8,
    /// Expansion J2.
    PB9: B 9,
    /// Expansion J2.
    PB10: B 10,
    /// Expansion J2.
    PB11: B 11,
    /// Expansion J2.
    PB12: B 12,
    /// Expansion J2.
    PB13: B 13,
    /// Expansion J2.
    PB14: B 14,
    /// Expansion J2.
    PB15: B 15,
}
pin_hw! {
    /// **Hardwired:** SWO (trace).
    PB3: B 3,
}

// Port C
pin_free! {
    PC1: C 1,
    PC3: C 3,
    PC4: C 4,
    PC5: C 5,
    PC7: C 7,
    PC9: C 9,
    PC10: C 10,
    PC11: C 11,
    PC12: C 12,
    PC13: C 13,
    PC14: C 14,
    PC15: C 15,
}
pin_hw! {
    /// **Hardwired:** FMC_WE (SDRAM).
    PC0: C 0,
    /// **Hardwired:** FMC_SDNE0 / CS (SDRAM).
    PC2: C 2,
    /// **Hardwired:** FMC_NWAIT (NAND).
    PC6: C 6,
    /// **Hardwired:** FMC_NCE (NAND).
    PC8: C 8,
}

// Port D
pin_free! {
    PD2: D 2,
    PD3: D 3,
    PD6: D 6,
    PD7: D 7,
    PD13: D 13,
}
pin_hw! {
    /// **Hardwired:** FMC_D2.
    PD0: D 0,
    /// **Hardwired:** FMC_D3.
    PD1: D 1,
    /// **Hardwired:** FMC_NOE (NAND).
    PD4: D 4,
    /// **Hardwired:** FMC_NWE (NAND).
    PD5: D 5,
    /// **Hardwired:** FMC_D13.
    PD8: D 8,
    /// **Hardwired:** FMC_D14.
    PD9: D 9,
    /// **Hardwired:** FMC_D15.
    PD10: D 10,
    /// **Hardwired:** FMC_CLE (NAND).
    PD11: D 11,
    /// **Hardwired:** FMC_ALE (NAND).
    PD12: D 12,
    /// **Hardwired:** FMC_D0.
    PD14: D 14,
    /// **Hardwired:** FMC_D1.
    PD15: D 15,
}

// Port E — full set including LEDs
pin_free! {
    /// Status LED **RUN** (D4 anode). Prefer [`crate::hal::gpio::led`].
    PE2: E 2,
    /// Status LED **DBG** (D5 anode). Prefer [`crate::hal::gpio::led`].
    PE3: E 3,
    /// Status LED **ERR** (D6 anode). Prefer [`crate::hal::gpio::led`].
    PE4: E 4,
    /// Status LED **PNC** (D7 anode). Prefer [`crate::hal::gpio::led`].
    PE5: E 5,
    /// NC / free.
    PE6: E 6,
}
pin_hw! {
    /// **Hardwired:** FMC_NBL0.
    PE0: E 0,
    /// **Hardwired:** FMC_NBL1.
    PE1: E 1,
    /// **Hardwired:** FMC_D4.
    PE7: E 7,
    /// **Hardwired:** FMC_D5.
    PE8: E 8,
    /// **Hardwired:** FMC_D6.
    PE9: E 9,
    /// **Hardwired:** FMC_D7.
    PE10: E 10,
    /// **Hardwired:** FMC_D8.
    PE11: E 11,
    /// **Hardwired:** FMC_D9.
    PE12: E 12,
    /// **Hardwired:** FMC_D10.
    PE13: E 13,
    /// **Hardwired:** FMC_D11.
    PE14: E 14,
    /// **Hardwired:** FMC_D12.
    PE15: E 15,
}

// Port F
pin_hw! {
    /// **Hardwired:** FMC_A0.
    PF0: F 0,
    /// **Hardwired:** FMC_A1.
    PF1: F 1,
    /// **Hardwired:** FMC_A2.
    PF2: F 2,
    /// **Hardwired:** FMC_A3.
    PF3: F 3,
    /// **Hardwired:** FMC_A4.
    PF4: F 4,
    /// **Hardwired:** FMC_A5.
    PF5: F 5,
    /// **Hardwired:** QSPI_IO3.
    PF6: F 6,
    /// **Hardwired:** QSPI_IO2.
    PF7: F 7,
    /// **Hardwired:** QSPI_IO0.
    PF8: F 8,
    /// **Hardwired:** QSPI_IO1.
    PF9: F 9,
    /// **Hardwired:** QSPI_CLK.
    PF10: F 10,
    /// **Hardwired:** FMC_SDNRAS.
    PF11: F 11,
    /// **Hardwired:** FMC_A6.
    PF12: F 12,
    /// **Hardwired:** FMC_A7.
    PF13: F 13,
    /// **Hardwired:** FMC_A8.
    PF14: F 14,
    /// **Hardwired:** FMC_A9.
    PF15: F 15,
}

// Port G
pin_free! {
    PG7: G 7,
    PG9: G 9,
    PG10: G 10,
    PG11: G 11,
    PG12: G 12,
    PG13: G 13,
    PG14: G 14,
}
pin_hw! {
    /// **Hardwired:** FMC_A10.
    PG0: G 0,
    /// **Hardwired:** FMC_A11.
    PG1: G 1,
    /// **Hardwired:** FMC_A12.
    PG2: G 2,
    /// **Hardwired:** NAND_WP.
    PG3: G 3,
    /// **Hardwired:** FMC_BA0.
    PG4: G 4,
    /// **Hardwired:** FMC_BA1.
    PG5: G 5,
    /// **Hardwired:** QSPI_NCS (board bodge to PG6).
    PG6: G 6,
    /// **Hardwired:** FMC_SDCLK.
    PG8: G 8,
    /// **Hardwired:** FMC_SDNCAS.
    PG15: G 15,
}

// Port H
pin_free! {
    PH1: H 1,
}
pin_hw! {
    /// **Hardwired:** HSE OSC_IN.
    PH0: H 0,
    /// **Hardwired:** display connector / LTDC candidate.
    PH2: H 2,
    /// **Hardwired:** display connector / LTDC candidate.
    PH3: H 3,
    /// **Hardwired:** display connector / LTDC candidate.
    PH4: H 4,
    /// **Hardwired:** display connector / LTDC candidate.
    PH5: H 5,
    /// **Hardwired:** display connector / LTDC candidate.
    PH6: H 6,
    /// **Hardwired:** FMC_SDCKE.
    PH7: H 7,
    /// **Hardwired:** display connector / LTDC candidate.
    PH8: H 8,
    /// **Hardwired:** display connector / LTDC candidate.
    PH9: H 9,
    /// **Hardwired:** display connector / LTDC candidate.
    PH10: H 10,
    /// **Hardwired:** display connector / LTDC candidate.
    PH11: H 11,
    /// **Hardwired:** display connector / LTDC candidate.
    PH12: H 12,
    /// **Hardwired:** display connector / LTDC candidate.
    PH13: H 13,
    /// **Hardwired:** display connector / LTDC candidate.
    PH14: H 14,
    /// **Hardwired:** display connector / LTDC candidate.
    PH15: H 15,
}

// Port I
pin_free! {
    PI0: I 0,
    PI1: I 1,
    PI2: I 2,
    PI3: I 3,
    PI4: I 4,
    PI5: I 5,
    PI6: I 6,
    PI7: I 7,
    PI8: I 8,
    PI9: I 9,
    PI10: I 10,
    PI12: I 12,
    PI13: I 13,
    PI14: I 14,
    PI15: I 15,
}
pin_hw! {
    /// **Hardwired:** QSPI_RES (NOR RESET#).
    PI11: I 11,
}

// Port J (bonded subset)
pin_free! {
    PJ6: J 6,
    PJ7: J 7,
    PJ8: J 8,
    PJ9: J 9,
    PJ10: J 10,
    PJ11: J 11,
}

// Port K (bonded subset)
pin_free! {
    PK0: K 0,
    PK1: K 1,
    PK2: K 2,
}
