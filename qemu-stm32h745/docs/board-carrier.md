# Carrier board configuration

Source: KiCad netlist `STM32H745 'Computer'.net` (2026-07-14), Revision 1.0.

## MCU

| Item | Value |
|------|--------|
| Part | **STM32H745BITx** (U1, LQFP208) — BIT package, 2 MiB flash |
| Primary core | Cortex-M7 (CM4 later) |
| HSE | 8 MHz (`616L3I008M00000R`) |

## Status LEDs (carrier)

| LED | Ref | MCU | Active |
|-----|-----|-----|--------|
| **RUN** | D4 | **PE2** | high → on |
| **DBG** | D5 | **PE3** | high → on |
| **ERR** | D6 | **PE4** | high → on |
| **PNC** | D7 | **PE5** | high → on |

QEMU opens a **second window** (`stm32h745-status-leds`) driven by these GPIO outputs.

Bootloader: DBG while bring-up → RUN if OS vectors OK → ERR blink + red panel if NOR image invalid.

## Debug — J4 (`Conn_ARM_JTAG_SWD_10`)

| Header pin | Net | MCU |
|------------|-----|-----|
| 1 VTref | 3V3 | — |
| 2 SWDIO/TMS | SWDIO | **PA13** |
| 3 GND | GND | — |
| 4 SWCLK/TCK | SWCLK | **PA14** |
| 5 GND | GND | — |
| 6 SWO/TDO | SWO | **PB3** |
| 7 KEY | NC | — |
| 8 NC/TDI | NC | — |
| 9 GNDDetect | GND | — |
| 10 nRESET | RESET | **NRST** (+ SW1) |

`BOOT0` pulled down via 10k (R1) → normal boot from internal flash.

## External NOR — IC3 `IS25LP01GJ-RMLE-TY`

| Item | Value |
|------|--------|
| Density | **1 Gbit = 128 MiB** (128M × 8) |
| Interface | QSPI / QPI |
| Package | 16-pin SOP 300 mil |
| QEMU map | `0x90000000`, size `0x08000000` |
| DCR.FSIZE | `26` (`2^(FSIZE+1)` bytes = 128 MiB) |

### QUADSPI pinmux (from netlist)

| Net | NOR pin | MCU pin | QUADSPI function (AF9) |
|-----|---------|---------|------------------------|
| QSPI_CLK | SCK | **PF10** | QUADSPI_CLK |
| QSPI_IO0 | SI / IO0 | **PF8** | QUADSPI_BK1_IO0 |
| QSPI_IO1 | SO / IO1 | **PF9** | QUADSPI_BK1_IO1 |
| QSPI_IO2 | WP# / IO2 | **PF7** | QUADSPI_BK1_IO2 |
| QSPI_IO3 | HOLD#/RESET# / IO3 | **PF6** | QUADSPI_BK1_IO3 |
| QSPI_RES | RESET# | **PI11** | GPIO (hold high after power-up) |
| **QSPI_NCS** | **CE#** | **⚠ NO MCU NODE** | **must be QUADSPI_BK1_NCS** |

### ⚠ Schematic fix: `QSPI_NCS` (enamel bodge is fine)

Netlist: **`QSPI_NCS` → IC3 CE# only** (no MCU pin).

**Bodge CE# → PG6** (QUADSPI_BK1_NCS, AF10). Firmware and QEMU both require
**PG6 AF10** before NOR is visible.

### QEMU fidelity

External memories appear in the CPU map only after the **same** steps as
hardware: GPIO clocks → pinmux (including PG6 + PI11 high) → AHB3 FMC/QSPI
clocks → FMC init sequence / QSPI memmap. Skipping any step = no map.

## External SDRAM — IC1 `AS4C32M16SB-7TCN`

| Item | Value |
|------|--------|
| Bus | FMC 16-bit |
| Size | 64 MiB |
| QEMU map | `0xC0000000` |

(Pinmux fully present on FMC_* nets — see netlist.)

## External NAND — IC2 `MX30LF4G28AD-TI-T`

FMC NAND (mass storage). Not used by stage-0 / XIP OS path yet.

## Display / I2C / UART

Still partially TBD in firmware; netlist has display connector PH6 and other IO.

## Flashing via SWD (after NCS is fixed)

See project README / `docs/flashing.md` summary:

1. SWD on **J4** programs **MCU internal flash** only → `bootloader` @ `0x08000000`.
2. ArmOS must land in **IS25LP01GJ** via QSPI (CubeProgrammer external loader, or a one-shot NOR burner in internal flash).
3. Reset with BOOT0=0 → bootloader → QSPI memmap → jump NOR @ `0x90000000`.

## QEMU machine properties

```
-machine stm32h745-carrier,os-image=ArmOS.elf
```

- `hse-frequency=8000000`
- `sdram-size=0x4000000` (64 MiB)
- `nor-size=0x8000000` (128 MiB — matches IS25LP01GJ)
