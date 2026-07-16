CROSS ?= arm-none-eabi-
CC    := $(CROSS)gcc

CFLAGS := -mcpu=cortex-m7 -mthumb -mfpu=fpv5-d16 -mfloat-abi=hard \
          -ffreestanding -nostdlib -O2 -Wall -Wextra
LDFLAGS := -T linker.ld -nostdlib -Wl,--gc-sections

all: uart_hello.elf ltdc_bars.elf

uart_hello.elf: uart_hello.c linker.ld
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ uart_hello.c

ltdc_bars.elf: ltdc_bars.c linker.ld
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ ltdc_bars.c

clean:
	rm -f uart_hello.elf ltdc_bars.elf *.bin

.PHONY: all clean
