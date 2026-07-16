//! Board-level storage ICs (MCU internal flash/SRAM are **bootloader-only**).
//!
//! | Medium | Interface | Size | Intended use |
//! |--------|-----------|------|----------------|
//! | SDRAM | FMC | 64 MiB | **OS main RAM** |
//! | NOR | QUADSPI | 128 MiB | **OS code XIP** + RO |
//! | NAND | FMC | 512 MiB | Mass storage / filesystem |
//!
//! Stage-0 bootloader: MCU flash + DTCM. NAND/QUADSPI drivers are stubs until
//! hardware init is implemented.

use crate::board::{NAND_BASE, NAND_SIZE, NOR_BASE, NOR_SIZE, SDRAM_BASE, SDRAM_SIZE};

/// Identity of a board storage device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageKind {
    /// Main system RAM (FMC SDRAM).
    Sdram,
    /// Parallel/FMC NAND flash.
    Nand,
    /// QUADSPI NOR flash.
    Nor,
}

/// Static description of a storage device (no I/O yet).
#[derive(Clone, Copy, Debug)]
pub struct StorageInfo {
    pub kind: StorageKind,
    pub name: &'static str,
    pub base: usize,
    pub size: usize,
    pub bus: &'static str,
}

/// All board storage devices the OS is allowed to use for data.
pub const BOARD_STORAGE: &[StorageInfo] = &[
    StorageInfo {
        kind: StorageKind::Sdram,
        name: "SDRAM",
        base: SDRAM_BASE,
        size: SDRAM_SIZE,
        bus: "FMC",
    },
    StorageInfo {
        kind: StorageKind::Nand,
        name: "NAND",
        base: NAND_BASE,
        size: NAND_SIZE,
        bus: "FMC",
    },
    StorageInfo {
        kind: StorageKind::Nor,
        name: "NOR",
        base: NOR_BASE,
        size: NOR_SIZE,
        bus: "QUADSPI",
    },
];

/// NAND chip (512 MiB) — placeholder for FMC NAND driver + FS.
pub struct NandDevice;

impl NandDevice {
    pub const fn info() -> StorageInfo {
        BOARD_STORAGE[1]
    }

    /// Future: reset, ID read, block erase, page program/read.
    pub fn init() -> Result<(), StorageError> {
        Err(StorageError::NotImplemented("FMC NAND driver"))
    }
}

/// QUADSPI NOR (128 MiB) — placeholder for memory-map / command mode driver.
pub struct NorDevice;

impl NorDevice {
    pub const fn info() -> StorageInfo {
        BOARD_STORAGE[2]
    }

    pub fn init() -> Result<(), StorageError> {
        Err(StorageError::NotImplemented("QUADSPI NOR driver"))
    }
}

#[derive(Debug)]
pub enum StorageError {
    NotImplemented(&'static str),
}
