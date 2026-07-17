//! Carrier-board configuration and memory policy.
//!
//! # Boot architecture
//!
//! ```text
//!  Power-on
//!     │
//!     ▼
//!  MCU internal flash @ 0x08000000     ← stage-0 bootloader only
//!     │  (uses MCU DTCM for its stack/statics)
//!     │  enables FMC + QUADSPI as needed
//!     ▼
//!  QUADSPI NOR @ 0x90000000 (XIP)      ← ArmOS image (this package's main bin)
//!     │  uses FMC SDRAM as main RAM
//!     ▼
//!  FMC SDRAM @ 0xC0000000 (64 MiB)     ← framebuffers + .data/.bss/heap/stack
//!  FMC NAND  @ 0x80000000 (512 MiB)    ← mass storage / filesystem
//! ```
//!
//! | Resource | Owner |
//! |----------|--------|
//! | MCU ROM (2 MiB internal flash) | Bootloader / factory firmware only |
//! | MCU RAM (DTCM, AXI SRAM, …) | Bootloader only (not OS heap) |
//! | **SDRAM 64 MiB** | **OS main RAM** |
//! | **NOR 128 MiB** | **OS code + RO (XIP)** |
//! | **NAND 512 MiB** | **OS mass storage** |
//!
//! GPIO map: [`gpio`].

#![cfg_attr(feature = "mps2", allow(dead_code))]

pub mod gpio;

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

/// External HSE (OSC_IN) — carrier crystal `616L3I008M00000R`.
pub const HSE_FREQ_HZ: u32 = 8_000_000;

/// Target system clock after PLL1 bring-up (HSE → 480 MHz on CM7).
///
/// QEMU `stm32h745-carrier` wires this as SYSCLK/cpuclk so SysTick and
/// virtual timers match the silicon operating point. Guest code should use
/// this (via `hal::clock`) rather than assuming reset HSI (64 MHz).
pub const SYSCLK_FREQ_HZ: u32 = 480_000_000;

// ---------------------------------------------------------------------------
// Carrier memory map (confirm against schematic)
// ---------------------------------------------------------------------------

/// FMC SDRAM bank1 — **OS main RAM** (64 MiB).
pub const SDRAM_BASE: usize = 0xC000_0000;
pub const SDRAM_SIZE: usize = 64 * 1024 * 1024;

/// FMC NAND — mass storage (512 MiB device).
pub const NAND_BASE: usize = 0x8000_0000;
pub const NAND_SIZE: usize = 512 * 1024 * 1024;

/// QUADSPI memory-mapped NOR — **OS execute-in-place** + RO data (128 MiB device).
pub const NOR_BASE: usize = 0x9000_0000;
pub const NOR_SIZE: usize = 128 * 1024 * 1024;

/// Stage-0 bootloader lives here (MCU internal flash). Not used by ArmOS itself.
pub const BOOTLOADER_FLASH_BASE: usize = 0x0800_0000;

// ---------------------------------------------------------------------------
// Framebuffer (in SDRAM)
// ---------------------------------------------------------------------------

pub const FB_WIDTH: u32 = 800;
pub const FB_HEIGHT: u32 = 480;
pub const FB_BPP: u16 = 32;
pub const FB_BYTES_PER_PIXEL: u32 = FB_BPP as u32 / 8;
pub const FB_STRIDE: u32 = FB_WIDTH * FB_BYTES_PER_PIXEL;
pub const FB_PIXELS: u32 = FB_WIDTH * FB_HEIGHT;
pub const FB_BYTES: u32 = FB_PIXELS * FB_BYTES_PER_PIXEL;
pub const FB_DOUBLE_BYTES: u32 = FB_BYTES * 2;

/// Bytes reserved at the start of SDRAM for both framebuffers (64 KiB aligned).
pub const FB_RESERVE_BYTES: usize = {
    let raw = FB_DOUBLE_BYTES as usize;
    (raw + 0xFFFF) & !0xFFFF
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scanout {
    Soft,
    Ltdc,
}

#[cfg(feature = "mps2")]
pub const FB_REGION_BASE: usize = 0x6000_0000;
#[cfg(feature = "mps2")]
pub const SCANOUT: Scanout = Scanout::Soft;
#[cfg(feature = "mps2")]
pub const MAIN_RAM_BASE: usize = FB_REGION_BASE + FB_RESERVE_BYTES;
#[cfg(feature = "mps2")]
pub const MAIN_RAM_SIZE: usize = 16 * 1024 * 1024 - FB_RESERVE_BYTES;

#[cfg(not(feature = "mps2"))]
pub const FB_REGION_BASE: usize = SDRAM_BASE;
#[cfg(not(feature = "mps2"))]
pub const SCANOUT: Scanout = Scanout::Ltdc;
#[cfg(not(feature = "mps2"))]
pub const MAIN_RAM_BASE: usize = SDRAM_BASE + FB_RESERVE_BYTES;
#[cfg(not(feature = "mps2"))]
pub const MAIN_RAM_SIZE: usize = SDRAM_SIZE - FB_RESERVE_BYTES;

/// Stack size at the top of main RAM (OS). Must match `linker.ld`.
pub const STACK_SIZE: usize = 64 * 1024;

// ---------------------------------------------------------------------------
// Panel timings
// ---------------------------------------------------------------------------

pub const PANEL_HSYNC: u16 = 48;
pub const PANEL_HBP: u16 = 40;
pub const PANEL_HFP: u16 = 40;
pub const PANEL_VSYNC: u16 = 1;
pub const PANEL_VBP: u16 = 31;
pub const PANEL_VFP: u16 = 13;
pub const PANEL_PIXEL_CLOCK_HZ: u32 = 29_200_000;

pub const BACKLIGHT_PWREN: Option<GpioPin> = None;
pub const BACKLIGHT_PWM: Option<GpioPin> = None;
pub const LTDC_PINS: &[GpioPin] = &[];

#[derive(Clone, Copy)]
pub enum GpioPort {
    A,
}

pub struct GpioPin {
    pub port: GpioPort,
    pub pin: u8,
    pub alternate: u8,
}
