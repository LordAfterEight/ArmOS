/* Legacy mps2-an500 + PSRAM stand-in for SDRAM experiments.
 * Prefer default build + stm32h745-carrier for real layout.
 *
 * Heap symbols required by the free-list allocator.
 */
ENTRY(Reset)

MEMORY
{
    FLASH (rx)  : ORIGIN = 0x00000000, LENGTH = 4096K
    RAM   (xrw) : ORIGIN = 0x20000000, LENGTH = 4096K
}

_estack = ORIGIN(RAM) + LENGTH(RAM);

/* PSRAM after FB_RESERVE 0x2F0000 — absolute, matches board mps2 MAIN_RAM */
_heap_start = 0x602F0000;
_heap_end   = 0x61000000; /* 16 MiB PSRAM end */

SECTIONS
{
    .vector_table ORIGIN(FLASH) :
    {
        KEEP(*(.vector_table))
    } > FLASH

    .text :
    {
        *(.text .text.*)
        *(.rodata .rodata.*)
        . = ALIGN(4);
    } > FLASH

    .data : AT(ADDR(.text) + SIZEOF(.text))
    {
        . = ALIGN(4);
        _sidata = LOADADDR(.data);
        _sdata = .;
        *(.data .data.*)
        . = ALIGN(4);
        _edata = .;
    } > RAM

    .bss :
    {
        . = ALIGN(4);
        _sbss = .;
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(4);
        _ebss = .;
    } > RAM

    /DISCARD/ :
    {
        *(.ARM.exidx*)
    }
}
