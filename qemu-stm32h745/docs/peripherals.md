# Peripheral matrix

| Block | Phase | Status | Guest-facing behaviour |
|-------|-------|--------|------------------------|
| ARMv7-M / NVIC / SysTick | 0 | Done | Stock QEMU Cortex-M7 |
| Flash / SRAM map | 0–1 | Done | ROM + RAM regions |
| RCC | 1–2 | Minimal | HSI/HSE/PLL1 ready; ENR/CFGR storage |
| GPIO | 1 | Minimal | MODER, AFR, ODR, BSRR, IDR |
| USART1 | 1 | Done (poll) | CR1/BRR/ISR/TDR/RDR → chardev |
| FMC / SDRAM | 3 | Gated | Bank1 init seq → map 64 MiB @ `0xC0000000` |
| QUADSPI / NOR | 3 | Gated | CR.EN + FMODE=memmap → map 128 MiB @ `0x90000000` |
| LTDC | 4 | Done (layer 1) | ARGB8888 scanout → QEMU console (~30 Hz) |
| I2C3 | 5 | Stub | Placeholder |
| TIM / PWM | 6 | Stub | Placeholder |
| ADC | 6 | Stub | Placeholder |
| CM4 / HSEM | 7 | Not started | |
| USB | 8 | Deferred | |

## USART1 notes

Register layout is the modern ST USART (same as L4/H7 PAC):

- `CR1` @ `0x00` — `UE`, `TE`, `RE`, IRQ enables  
- `BRR` @ `0x0C`  
- `ISR` @ `0x1C` — `TXE`, `TC`, `RXNE`, `TEACK`, `REACK`  
- `TDR` @ `0x28`, `RDR` @ `0x24`  

IRQ: NVIC **37**.

## RCC notes

- Reset: HSI on + ready.  
- `hse-frequency` SoC property (carrier sets **8_000_000**): enables `HSERDY` when `HSEON` is written.  
- PLL1 lock is immediate when `PLL1ON` is set (simplified).
