/*
 * ArmOS — runs from QUADSPI NOR (XIP), uses FMC SDRAM as main RAM.
 *
 *   Code/RO:  NOR @ 0x90000000  (128 MiB device; image uses a small slice)
 *   Main RAM: SDRAM after framebuffers
 *               [0xC0000000, 0xC02F0000)  FB double-buffer (not in linker)
 *               [0xC02F0000, 0xC4000000)  .data .bss heap + stack at top
 *
 * MCU internal flash holds the *bootloader* only (see linker-boot.ld).
 * MCU internal SRAM is reserved for the bootloader — not used here.
 *
 * FB_RESERVE = 0x2F0000 — keep in sync with board::FB_RESERVE_BYTES
 */
ENTRY(Reset)

MEMORY
{
    /* QUADSPI memory-mapped NOR window */
    NOR   (rx)  : ORIGIN = 0x90000000, LENGTH = 128M
    /* MAIN_RAM_BASE = 0xC0000000 + 0x2F0000; length = 64M - 0x2F0000 */
    RAM   (xrw) : ORIGIN = 0xC02F0000, LENGTH = 0x3D10000
}

/* 64 KiB stack at top of SDRAM — matches board::STACK_SIZE */
_estack = ORIGIN(RAM) + LENGTH(RAM);
_stack_limit = _estack - 64K;

SECTIONS
{
    .vector_table ORIGIN(NOR) :
    {
        KEEP(*(.vector_table))
    } > NOR

    .text :
    {
        *(.text .text.*)
        *(.rodata .rodata.*)
        . = ALIGN(4);
    } > NOR

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
        . = ALIGN(8);
        _ebss = .;
    } > RAM

    _heap_start = _ebss;
    _heap_end = _stack_limit;

    /DISCARD/ :
    {
        *(.ARM.exidx*)
    }
}
