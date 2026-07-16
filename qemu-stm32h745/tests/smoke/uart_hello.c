/*
 * Minimal guest: enable GPIOA + USART1, print "OK\r\n", then WFI.
 * Build: see Makefile (needs arm-none-eabi-gcc).
 */

#include <stdint.h>

#define RCC_BASE     0x58024400u
#define GPIOA_BASE   0x58020000u
#define USART1_BASE  0x40011000u

#define REG32(a) (*(volatile uint32_t *)(a))

#define RCC_AHB4ENR  REG32(RCC_BASE + 0xE0u)
#define RCC_APB2ENR  REG32(RCC_BASE + 0xF0u)

#define GPIOA_MODER  REG32(GPIOA_BASE + 0x00u)
#define GPIOA_AFRH   REG32(GPIOA_BASE + 0x24u)

#define USART1_CR1   REG32(USART1_BASE + 0x00u)
#define USART1_BRR   REG32(USART1_BASE + 0x0Cu)
#define USART1_ISR   REG32(USART1_BASE + 0x1Cu)
#define USART1_TDR   REG32(USART1_BASE + 0x28u)

#define USART_ISR_TXE (1u << 7)

static void uart_putc(char c)
{
    while ((USART1_ISR & USART_ISR_TXE) == 0) {
    }
    USART1_TDR = (uint32_t)(uint8_t)c;
}

static void uart_puts(const char *s)
{
    while (*s) {
        uart_putc(*s++);
    }
}

void Reset_Handler(void);
void Default_Handler(void);

/* Vector table in flash: SP @ DTCM top, Reset handler. */
__attribute__((section(".vectors")))
const uint32_t vectors[] = {
    0x20020000u,                 /* initial MSP: end of 128 KiB DTCM */
    (uint32_t)Reset_Handler,
};

void Default_Handler(void)
{
    for (;;) {
    }
}

void Reset_Handler(void)
{
    /* GPIOA + USART1 clocks */
    RCC_AHB4ENR |= (1u << 0);
    RCC_APB2ENR |= (1u << 4);

    /* PA9 AF7 (USART1_TX): MODER9 = AF, AFRH pin9 = 7 */
    GPIOA_MODER = (GPIOA_MODER & ~(3u << 18)) | (2u << 18);
    GPIOA_AFRH = (GPIOA_AFRH & ~(0xFu << 4)) | (7u << 4);

    /* 115200-ish at 64 MHz PCLK: BRR = 64000000/115200 ≈ 555 */
    USART1_BRR = 555u;
    USART1_CR1 = (1u << 0) | (1u << 3); /* UE | TE */

    uart_puts("OK\r\n");

    for (;;) {
        __asm volatile ("wfi");
    }
}
