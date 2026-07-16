# Memory map — STM32H745 carrier (QEMU model)

| Region | Base | Size | Notes |
|--------|------|------|--------|
| Flash alias (boot) | `0x00000000` | 2 MiB | Alias of main flash for reset vectors |
| Flash | `0x08000000` | 2 MiB | `-kernel` load region |
| DTCM | `0x20000000` | 128 KiB | Default ArmOS stack/data |
| AXI SRAM | `0x24000000` | 512 KiB | |
| SRAM1 | `0x30000000` | 128 KiB | |
| SRAM2 | `0x30020000` | 128 KiB | |
| SRAM3 | `0x30040000` | 32 KiB | |
| SRAM4 | `0x38000000` | 64 KiB | |
| FMC SDRAM bank1 | `0xC0000000` | 64 MiB | AS4C32M16SB; FB base for LTDC |

## Peripherals (implemented subset)

| Device | Base | Model |
|--------|------|--------|
| USART1 | `0x40011000` | `stm32h7-usart` |
| GPIOA–K | `0x58020000` + `0x400 * n` | `stm32h7-gpio` |
| RCC | `0x58024400` | `stm32h7-rcc` |

Unimplemented windows (PWR, EXTI, SYSCFG, FLASH IF, FMC, LTDC, I2C3, TIMx, ADC, DMA, …) are stubs via `create_unimplemented_device` so guest probes do not abort.
