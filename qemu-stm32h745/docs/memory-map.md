# Memory map — STM32H745 carrier

## Boot architecture

```text
 Power-on / reset
        │
        ▼
 MCU internal flash @ 0x08000000     stage-0 bootloader (DTCM stack)
        │  optional: clocks, FMC, QUADSPI init
        ▼
 QUADSPI NOR @ 0x90000000 (XIP)      ArmOS image (vectors + .text + .rodata)
        │
        ▼
 FMC SDRAM @ 0xC0000000              OS main RAM
 FMC NAND  @ 0x80000000              mass storage (FS later)
```

| Resource | Who uses it |
|----------|-------------|
| MCU ROM (2 MiB `@ 0x08000000`) | **Bootloader only** |
| MCU RAM (DTCM / AXI SRAM / …) | **Bootloader only** |
| **NOR 128 MiB** `@ 0x90000000` | **OS code (XIP)** |
| **SDRAM 64 MiB** `@ 0xC0000000` | **OS main RAM** |
| **NAND 512 MiB** `@ 0x80000000` | **OS storage** |

## SDRAM layout (OS main RAM)

```text
0xC0000000  +----------------------------+
            | FB0 + FB1                  |  FB_RESERVE = 0x2F0000
0xC02F0000  +----------------------------+  linker RAM / MAIN_RAM_BASE
            | .data / .bss               |
            | free-list heap             |
            | …                          |
            | ← stack (64 KiB)           |
0xC4000000  +----------------------------+
```

## QEMU load (hardware-gated)

At reset, only MCU flash + internal SRAMs are in the CPU map.
SDRAM and NOR exist as external ICs but are **invisible** until the bootloader
runs the real init sequences (FMC bank1 + QUADSPI memory-map).

```bash
qemu-system-arm \
  -machine stm32h745-carrier,os-image=ArmOS.elf \  # factory-program NOR array
  -kernel bootloader.elf \                          # MCU flash @ 0x08000000
  -serial mon:stdio -display sdl
```

| Step | What happens |
|------|----------------|
| Machine start | NOR array pre-programmed from `os-image` (not CPU-mapped) |
| Bootloader | RCC AHB3ENR → FMC CLOCK/PALL/AUTOREF/LOAD → QSPI FMODE=memmap+EN |
| After init | SDRAM @ `0xC0000000`, NOR XIP @ `0x90000000` visible |
| Handoff | Bootloader jumps to NOR vector table |

`cargo run --release` does this automatically via `scripts/qemu-run.sh`.
