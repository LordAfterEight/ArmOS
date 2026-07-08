/* STM32H745BIT6 — Cortex-M7, 2 MiB flash, 128 KiB DTCM
 * Flash base: 0x08000000
 */
ENTRY(Reset)

MEMORY
{
    FLASH (rx)  : ORIGIN = 0x08000000, LENGTH = 2048K
    RAM   (xrw) : ORIGIN = 0x20000000, LENGTH = 128K
}

_estack = ORIGIN(RAM) + LENGTH(RAM);

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