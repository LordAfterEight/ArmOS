# Carrier board configuration

## MCU

- **STM32H745BIT6** (2 MiB flash, LQFP208)
- Primary core: **Cortex-M7** (CM4 later)

## Clocks

| Source | Frequency | Wiring |
|--------|-----------|--------|
| HSE | 8 MHz | Oscillator into **OSC_IN (PH0, pin 35)** |
| HSI | 64 MHz | Internal (reset default SYSCLK in model) |

## External memory

| Part | Bus | Size | QEMU mapping |
|------|-----|------|----------------|
| AS4C32M16SB-7TCN | FMC, 16-bit | 64 MiB | `0xC0000000` |

## Display

| Item | Value |
|------|--------|
| Interface | LTDC parallel RGB + I2C touch |
| Panel class | SL-TFT7 / Powertip PH800480T013 |
| Resolution | 800 × 480, ARGB/XRGB8888 |
| FB base (guest) | `0xC0000000` (in SDRAM) |
| LTDC pinmux | TBD (schematic) |
| Backlight | TBD |

## I2C3

| Signal | Pin |
|--------|-----|
| SDA | PC9 |
| SCL | PA8 |
| SMBA | PA9 |

Touch controller: soft stub until part is chosen.

## Console UART

Default firmware: **USART1**, PA9 TX / PA10 RX AF7.

**Conflict:** PA9 is also I2C3_SMBA. Prefer leaving SMBA unused or move console UART when the schematic freezes.

## Machine properties

```
-machine stm32h745-carrier
```

SoC properties (set by board code):

- `hse-frequency=8000000`
- `sdram-size=0x4000000` (64 MiB)
