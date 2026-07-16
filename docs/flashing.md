# Flashing ArmOS to the carrier (SWD + IS25LP01GJ)

## Hardware map (from netlist)

| Image | Device | Address |
|-------|--------|---------|
| `bootloader` | STM32H745 internal flash | `0x08000000` |
| `ArmOS` | **IS25LP01GJ-RMLE-TY** (128 MiB QSPI NOR) | XIP window `0x90000000` |

Debug connector **J4** (10-pin ARM SWD):

| Pin | Signal | MCU |
|-----|--------|-----|
| 1 | VTref (3V3) | — |
| 2 | SWDIO | PA13 |
| 4 | SWCLK | PA14 |
| 6 | SWO | PB3 |
| 10 | nRESET | NRST |
| 3,5,9 | GND | — |

## Prerequisite: fix `QSPI_NCS`

The netlist has **CE# of the NOR floating off-chip only** — no MCU connection.
Connect `QSPI_NCS` → **PG6** (QUADSPI_BK1_NCS, AF10) or another BK1_NCS pin, then rebuild the board / bodgewire.

Until that is fixed, **neither production loaders nor a NOR burner can talk to the flash.**

## 1. Build images

```bash
cd ArmOS
cargo build --release --bin bootloader --bin ArmOS
```

Outputs:

```text
target/thumbv7em-none-eabihf/release/bootloader
target/thumbv7em-none-eabihf/release/ArmOS
```

Optional binaries:

```bash
rustup component add llvm-tools
cargo install cargo-binutils
cargo objcopy --release --bin bootloader -- -O binary bootloader.bin
cargo objcopy --release --bin ArmOS -- -O binary ArmOS.bin
```

`ArmOS.bin` is the contiguous LMA image starting at NOR base (vectors first).  
Confirm with `readelf -l …/ArmOS` that `PhysAddr` values sit in `0x90000000…`.

## 2. Flash bootloader via SWD (always step one)

### probe-rs

```bash
cargo install probe-rs-tools

probe-rs list
probe-rs download \
  --chip STM32H745BITx \
  --binary-format elf \
  target/thumbv7em-none-eabihf/release/bootloader
probe-rs reset --chip STM32H745BITx
```

(Use the exact chip string from `probe-rs chip list | grep -i H745` if the suffix differs.)

### OpenOCD + ST-Link

```bash
openocd -f interface/stlink.cfg -f target/stm32h7x.cfg \
  -c "program target/thumbv7em-none-eabihf/release/bootloader verify reset exit"
```

### STM32CubeProgrammer

Connect ST-Link on J4 → download `bootloader` ELF to **`0x08000000`**.

After this alone, the bootloader will init FMC/QSPI and jump to NOR. If NOR is empty you will HardFault — expected.

## 3. Program ArmOS into IS25LP01GJ

SWD does **not** write external NOR by itself. Pick one path:

### A. STM32CubeProgrammer external loader (preferred when available)

1. Install / build an **external loader** (`.stldr`) for:
   - STM32H745 + QUADSPI bank1
   - Pins: **PF6–PF10** (IO3..CLK), **NCS = your fixed pin (PG6 recommended)**
   - Flash: **ISSI IS25LP01G** family (1 Gbit, 4-byte address mode)
2. In CubeProgrammer: *External loaders* → select that loader → download `ArmOS.bin` at **`0x90000000`**.

ISSI parts use standard SPI NOR commands (READ 0x03 / QUAD 0x6B / 4-byte addr variants, JEDEC 0x9F, sector erase 0x20 / 0xD8, page program 0x02). Loaders for similar 1 Gbit ISSI / Winbond parts are often adaptable once pinmux matches.

### B. One-shot NOR burner (always works with SWD)

1. Build a small firmware linked to **internal flash** that:
   - holds `ArmOS.bin` as a byte array (or streams it over USART)
   - enables RCC QUADSPI + GPIO pinmux (PF6–10, NCS, deassert PI11 reset)
   - puts QSPI in **indirect mode**
   - issues Write Enable → erase → page program over the whole image
2. SWD-flash that helper, run once, then SWD-flash the real **bootloader** again.
3. Reset → XIP.

This is the recommended bring-up path until a `.stldr` exists for this exact board.

## Boot failure UI

If NOR vectors are missing/invalid, the bootloader **does not jump**. It:

1. Drives **ERR** (PE4) with a slow blink (and leaves RUN off)
2. Fills the LTDC framebuffer with a **red striped** full-screen pattern
3. Enables LTDC so the main panel is obviously “failed load”

In QEMU, also watch the **status LED** window (RUN/DBG/ERR/PNC).

## 4. Boot sequence on silicon

```text
Reset (BOOT0=0)
  → CM7 vectors from internal flash (bootloader)
  → RCC AHB3ENR: FMCEN | QSPIEN
  → GPIO AF for QSPI + FMC SDRAM
  → FMC bank1: CLOCK → PALL → AUTOREF → LOAD_MODE
  → QUADSPI: DCR.FSIZE=26, CCR FMODE=memmap, CR.EN
  → deassert PI11 (NOR RESET#) if used as GPIO
  → SCB->VTOR = 0x90000000; MSP/PC from NOR; bx Reset
  → ArmOS runs XIP from IS25LP01GJ, RAM in AS4C32M16SB SDRAM
```

## 5. Bootloader = silicon path (also required in QEMU)

`src/bin/bootloader.rs` performs the **full** sequence. QEMU will **not** map
SDRAM/NOR if any step is skipped:

1. RCC AHB4ENR — GPIO C/D/E/F/G/H/I clocks  
2. GPIO AF pinmux — FMC AF12 bus + QSPI PF6–10 / PG6 + PI11 high  
3. RCC AHB3ENR — FMCEN | QSPIEN  
4. FMC bank1 CLOCK → PALL → AUTOREF → LOAD_MODE  
5. QUADSPI DCR.FSIZE=26, CCR memmap, CR.EN  
6. Jump to `0x90000000`

Still silicon-only details (not yet in model): ISSI 4-byte address enable if
needed for the full 128 MiB, exact SDRAM timing numbers, QSPI read command
fields in CCR for true XIP protocol.

## Schematic notes

| Item | Action |
|------|--------|
| `QSPI_NCS` open | Bodge CE# → **PG6** (bootloader + QEMU already assume PG6) |
| `FMC_CKE` on **PH7** | ST AF = `FMC_SDCKE1`, while `FMC_CS` on PC2 = `FMC_SDNE0` — bank pairing is inconsistent on paper. Verify on the bench; may need CKE on SDCKE0 (e.g. PC3/PH2) |

## QEMU vs board

| | QEMU `stm32h745-carrier` | Board |
|--|-------------------------|--------|
| NOR size | 128 MiB | IS25LP01GJ 128 MiB ✓ |
| NOR content | `os-image=` factory array | SWD + QSPI program |
| RCC gates | **required** | required |
| GPIO pinmux | **required (carrier nets)** | required |
| FMC init seq | **required** | required |
| QSPI memmap | **required** | required |
| NCS | model expects **PG6** | bodge NCS→PG6 |
