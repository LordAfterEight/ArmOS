/*
 * Stage-0 bootloader — MCU internal flash + DTCM only.
 *
 * Responsibilities:
 *   - Bring-up clocks / FMC / QUADSPI as needed (stubs for now under QEMU)
 *   - Jump to ArmOS vector table in NOR (XIP @ 0x90000000)
 *
 * Must not depend on SDRAM, NAND, or OS heap.
 */
ENTRY(Reset)

MEMORY
{
    FLASH (rx)  : ORIGIN = 0x08000000, LENGTH = 128K
    RAM   (xrw) : ORIGIN = 0x20000000, LENGTH = 64K
}

_estack = ORIGIN(RAM) + LENGTH(RAM);

/* Unused by bootloader, but keep symbols defined if linked against stubs */
_heap_start = .;
_heap_end = .;

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
