# How the custom QEMU machine works

## What “a QEMU machine” is

When you run:

```bash
qemu-system-arm -machine stm32h745-carrier -kernel ArmOS.elf
```

QEMU does not load a generic “ARM CPU in empty space.” It constructs a **virtual board**: one address space, a CPU, memories, and MMIO devices wired like the STM32H745 carrier board.

That construction is split into two layers:

| Layer | Type name | Job |
|--------|-----------|-----|
| **Machine** | `stm32h745-carrier` | Board: clocks, SDRAM size, HSE, load the ELF |
| **SoC** | `stm32h745-soc` | Chip: CM7, flash/SRAM map, RCC/GPIO/USART/LTDC |

The machine creates the SoC, configures board properties, then loads firmware. After that, almost everything interesting is: **guest code runs on the emulated CM7 and touches memory/MMIO; device models react.**

---

## Startup sequence

### 1. Machine init (`stm32h745_carrier`)

1. Create a fixed **SYSCLK** object (default **64 MHz**, HSI-class).
2. Create `stm32h745-soc` and set:
   - `hse-frequency = 8_000_000` (carrier crystal)
   - `sdram-size = 64 MiB`
3. Connect SYSCLK into the SoC and **realize** it (build the whole chip).
4. Call **`armv7m_load_kernel`**: parse the ELF, place segments into flash, prepare Cortex-M reset (MSP/PC from the vector table).

### 2. SoC realize (`stm32h745_soc`)

This is where the virtual MCU is actually built:

1. **Memories** are added to the system address space (flash, DTCM, AXI SRAM, SDRAM, …).
2. **ARMv7-M** container is realized: Cortex-M7 + NVIC (150 IRQs), clocks attached.
3. **Peripherals** are realized and **mapped** at STM32 base addresses.
4. **IRQs** from devices are connected to NVIC lines (e.g. USART1 → 37, LTDC → 88).
5. Remaining holes get **unimplemented** stubs so stray guest accesses do not abort QEMU.

After realize, the guest is reset and starts executing like a bare-metal H7.

---

## Address space: the guest’s view of the world

Everything the firmware does is loads/stores (or fetches) in **one physical address space**.

```text
0x0000_0000  flash alias (boot)     ──┐
0x0800_0000  main flash 2 MiB       ──┴── same ROM contents
0x2000_0000  DTCM 128 KiB              stack / data (ArmOS linker)
0x2400_0000  AXI SRAM 512 KiB
0x3000_0000  SRAM1/2/3 …
0x4001_1000  USART1 MMIO
0x5000_1000  LTDC MMIO
0x5802_0000  GPIOA …
0x5802_4400  RCC
0xC000_0000  FMC SDRAM 64 MiB          framebuffers + heap
```

**Flash alias at 0:** On a real H7, boot from flash makes the vector table available at address 0. This machine maps the same flash both at `0x08000000` and as an alias at `0`. That way the core can take the reset vector the normal Cortex-M way.

**SDRAM at `0xC0000000`:** External DRAM behind FMC. **Not** mapped at reset.
The bootloader must run the bank-1 sequence (CLOCK → PALL → AUTOREF → LOAD_MODE)
before the guest can use it. Timing numbers are simplified (commands complete
instantly; no refresh stalls), but the visibility gate is real.

**NOR at `0x90000000`:** External QUADSPI flash. Array is factory-programmed
from machine property `os-image=…`. **Not** in the CPU map until
`QUADSPI_CR.EN` + `CCR.FMODE=memory-mapped`.

QEMU implements this with **MemoryRegions** (ROM, RAM, MMIO) registered into the system memory map. A guest store to `0x40011028` does not hit “RAM”; it hits the USART device’s MMIO ops.

---

## CPU execution loop (simplified)

Once running:

1. TCG translates guest Thumb instructions and executes them.
2. Instruction fetch comes from flash (or the flash alias).
3. Loads/stores go through the memory core:
   - **RAM/ROM** → host buffer
   - **MMIO** → C `read`/`write` callbacks on that device
4. Exceptions/IRQs go through the **NVIC** model (stock QEMU ARMv7-M).
5. SysTick / virtual time advance with QEMU’s clock; devices can schedule timers (LTDC refresh does this).

There is no separate “simulator for the OS.” The firmware binary **is** the guest; the machine only supplies the silicon-shaped world it expects.

---

## How each major device works

### RCC (`0x58024400`)

Firmware writes bits like “enable GPIOA clock” / “enable USART1 clock.”

The model:

- Holds a register image.
- On **CR**: synthesizes ready flags (HSI always, HSE if the board set `hse-frequency`, PLL “locks” instantly when turned on).
- On **ENR / CFGR**: mostly stores what was written so software can read it back.

It does **not** yet recompute real bus frequencies for every peripheral. It exists so guest clock-enable sequences succeed and do not hard-fault on missing registers.

### GPIO (ports A–K)

Firmware sets mode/alternate function for USART pins.

The model stores `MODER` / `AFR` / `ODR` / etc. It does **not** route a physical TX pin into the UART. The UART is a separate MMIO block; pinmux is bookkeeping so guest init looks real.

### USART1 (`0x40011000`)

This is a full **I/O bridge** to the host.

Guest (simplified):

```text
enable clocks → configure GPIO AF → set BRR/CR1 →
poll ISR.TXE → write character to TDR
```

On `TDR` write, the device model calls QEMU’s **character backend** (`-serial mon:stdio`). That byte appears in the host terminal.

Host → guest is the reverse: typed input can set `RDR` and `RXNE` if RX is enabled.

IRQ line 37 is wired if TXE/RXNE interrupts are enabled; typical bring-up uses polling.

Register layout is the **H7/L4-style** USART (`CR1` at 0, `ISR` at `0x1C`, `TDR` at `0x28`), not the older F1/F2 layout.

### LTDC (`0x50001000`)

This is a **framebuffer scanner**, not a GPU.

Guest programs:

- Panel timings (`SSCR` / `BPCR` / `AWCR` / …)
- Layer 1: format (ARGB8888), pitch, line count, **CFBAR** = physical address of the front buffer in SDRAM
- Enable layer + LTDC, then **SRCR** reload when swapping buffers

The model, periodically (~30 Hz) or on reload:

1. Checks LTDC + layer enabled and `CFBAR ≠ 0`
2. Computes active width/height from timing registers (or pitch/lines fallback)
3. Reads lines from guest SDRAM into a host **DisplaySurface**
4. Pushes that surface to the QEMU graphic console → **SDL window**

So:

```text
Guest pixels in SDRAM  →  LTDC MMIO says “scan this address”
                       →  QEMU copies to host window
```

Double-buffering works because the guest draws in one SDRAM buffer and only points **CFBAR** at the finished one on present.

---

## Putting it together: one firmware boot

```text
QEMU builds machine + SoC
        │
        ▼
Load ELF into flash @ 0x08000000
        │
        ▼
CM7 reset: SP/PC from vectors
        │
        ▼
Guest: copy .data, zero .bss, enable FPU
        │
        ▼
USART bring-up via RCC + GPIO + USART1 ──► serial text on host
        │
        ▼
Clear framebuffers in SDRAM @ 0xC0000000
Program LTDC layer → CFBAR = front buffer
        │
        ▼
UI draw into back buffer in SDRAM
present() → new CFBAR + reload
        │
        ▼
LTDC model copies FB → SDL window
```

From the guest’s point of view it is talking to an STM32H745. From QEMU’s point of view it is executing translated machine code against a hand-written C model of a few peripherals and a memory map.

---

## What is “real” vs simplified

| Aspect | Behavior in this machine |
|--------|---------------------------|
| Instruction set / NVIC / SysTick | Real QEMU Cortex-M7 model |
| Address map for used regions | Matches H745 / carrier choices |
| USART console | Functionally real enough for polling TX/RX |
| LTDC + SDRAM framebuffer | Functionally real enough for ARGB8888 UI |
| Clocks / baud / pixel clock | Approximate or ignored for timing |
| FMC SDRAM controller | Gated: RCC FMCEN + full FMC AF pinmux + bank1 init |
| QUADSPI NOR | Gated: RCC QSPIEN + QSPI AF/NCS/RESET# + CR.EN + memmap |
| Pins, DMA, I2C, USB, CM4 | Not really modeled (stubs or absent) |

The machine **works** by giving the firmware the **same register and memory contracts** it uses on hardware for the paths that matter (boot, serial, display), while omitting silicon detail that does not affect those paths yet.

---

## One-sentence summary

**`stm32h745-carrier` is a QEMU board that instantiates a Cortex-M7, maps H745-like flash/SRAM/SDRAM, and attaches small C devices at real MMIO addresses so the firmware’s loads/stores become UART bytes and pixels on the host—without needing a different “QEMU-only” firmware layout.**

---

## Source map (for readers of the tree)

| File | Role |
|------|------|
| `qemu/hw/arm/stm32h745_carrier.c` | Machine: clocks, properties, kernel load |
| `qemu/hw/arm/stm32h745_soc.c` | SoC: memory map, device realize, IRQ wiring |
| `qemu/include/hw/arm/stm32h745_soc.h` | Addresses, sizes, SoC state |
| `qemu/hw/misc/stm32h7_rcc.c` | RCC model |
| `qemu/hw/gpio/stm32h7_gpio.c` | GPIO model |
| `qemu/hw/char/stm32h7_usart.c` | USART1 ↔ host serial |
| `qemu/hw/display/stm32h7_ltdc.c` | LTDC ↔ host window |
