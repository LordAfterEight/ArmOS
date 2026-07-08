// Ensure handler symbols are linked for global_asm.
use super::reset::{DefaultHandler, Reset};
#[used]
static _VECTOR_SYMBOLS: (unsafe extern "C" fn() -> !, extern "C" fn() -> !) =
    (Reset, DefaultHandler);

// Vector table via global_asm avoids const-eval pointer casts (Rust 2024).
// Do not add +1 here — the linker applies the Thumb bit via relocation.
core::arch::global_asm!(
    r#"
    .section .vector_table, "a"
    .word _estack
    .word Reset
    .word DefaultHandler  /* NMI */
    .word DefaultHandler  /* HardFault */
    .word DefaultHandler  /* MemManage */
    .word DefaultHandler  /* BusFault */
    .word DefaultHandler  /* UsageFault */
    .word 0
    .word 0
    .word 0
    .word 0
    .word DefaultHandler  /* SVCall */
    .word DefaultHandler  /* DebugMon */
    .word 0
    .word DefaultHandler  /* PendSV */
    .word DefaultHandler  /* SysTick */
    .rept 149
    .word DefaultHandler
    .endr
    "#,
);