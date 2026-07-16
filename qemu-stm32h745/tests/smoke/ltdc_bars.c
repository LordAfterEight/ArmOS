/*
 * LTDC color bars on stm32h745-carrier (FB @ 0xC0000000).
 * Build: make ltdc_bars.elf
 */
#include <stdint.h>

#define RCC_BASE  0x58024400u
#define LTDC_BASE 0x50001000u
#define FB_BASE   0xC0000000u
#define WIDTH     800u
#define HEIGHT    480u
#define REG32(a)  (*(volatile uint32_t *)(a))

void Reset_Handler(void);

__attribute__((section(".vectors")))
const uint32_t vectors[] = {
    0x20020000u,
    (uint32_t)Reset_Handler,
};

void Reset_Handler(void)
{
    uint32_t x, y;
    uint32_t hsync = 48, hbp = 40, hfp = 40;
    uint32_t vsync = 1, vbp = 31, vfp = 13;
    uint32_t hsw, vsh, ahbp, avbp, aaw, aah, totalw, totalh;

    REG32(RCC_BASE + 0xE4) |= (1u << 3); /* APB3ENR.LTDCEN */

    for (y = 0; y < HEIGHT; y++) {
        for (x = 0; x < WIDTH; x++) {
            uint32_t c;
            if (x < WIDTH / 3) {
                c = 0x00FF0000u;
            } else if (x < 2u * WIDTH / 3) {
                c = 0x0000FF00u;
            } else {
                c = 0x000000FFu;
            }
            ((volatile uint32_t *)FB_BASE)[y * WIDTH + x] = c;
        }
    }

    hsw = hsync - 1;
    vsh = vsync - 1;
    ahbp = hsync + hbp - 1;
    avbp = vsync + vbp - 1;
    aaw = hsync + hbp + WIDTH - 1;
    aah = vsync + vbp + HEIGHT - 1;
    totalw = hsync + hbp + WIDTH + hfp - 1;
    totalh = vsync + vbp + HEIGHT + vfp - 1;

    REG32(LTDC_BASE + 0x08) = (hsw << 16) | vsh;
    REG32(LTDC_BASE + 0x0C) = (ahbp << 16) | avbp;
    REG32(LTDC_BASE + 0x10) = (aaw << 16) | aah;
    REG32(LTDC_BASE + 0x14) = (totalw << 16) | totalh;
    REG32(LTDC_BASE + 0x2C) = 0;
    REG32(LTDC_BASE + 0x88) = ((hsync + hbp + WIDTH - 1) << 16) | (hsync + hbp);
    REG32(LTDC_BASE + 0x8C) = ((vsync + vbp + HEIGHT - 1) << 16) | (vsync + vbp);
    REG32(LTDC_BASE + 0x94) = 0;
    REG32(LTDC_BASE + 0x98) = 0xFF;
    REG32(LTDC_BASE + 0xA0) = 0x405;
    REG32(LTDC_BASE + 0xAC) = FB_BASE;
    REG32(LTDC_BASE + 0xB0) = ((WIDTH * 4) << 16) | ((WIDTH * 4) + 3);
    REG32(LTDC_BASE + 0xB4) = HEIGHT;
    REG32(LTDC_BASE + 0x84) = 1;
    REG32(LTDC_BASE + 0x24) = 1;
    REG32(LTDC_BASE + 0x18) = 1;

    for (;;) {
        __asm volatile ("wfi");
    }
}
