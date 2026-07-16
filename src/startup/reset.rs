unsafe extern "C" {
    static _sidata: u32;
    static mut _sdata: u32;
    static _edata: u32;
    static mut _sbss: u32;
    static _ebss: u32;
}

unsafe extern "Rust" {
    fn main() -> !;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Reset() -> ! {
    unsafe {
        pre_init();
        main();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn DefaultHandler() -> ! {
    loop {}
}

unsafe fn pre_init() {
    unsafe {
        // OS image executes from QUADSPI NOR; keep VTOR on the NOR vector table
        // after the bootloader handoff (idempotent if already set).
        set_vtor(0x9000_0000);
        copy_data();
        zero_bss();
        enable_fpu();
    }
}

unsafe fn set_vtor(addr: u32) {
    const SCB_VTOR: *mut u32 = 0xE000_ED08 as *mut u32;
    unsafe {
        core::ptr::write_volatile(SCB_VTOR, addr);
    }
}

unsafe fn copy_data() {
    let mut dst = core::ptr::addr_of_mut!(_sdata) as usize;
    let mut src = core::ptr::addr_of!(_sidata) as usize;
    let end = core::ptr::addr_of!(_edata) as usize;

    while dst < end {
        unsafe {
            core::ptr::write_volatile(
                dst as *mut u32,
                core::ptr::read_volatile(src as *const u32),
            );
        }
        dst = dst.wrapping_add(4);
        src = src.wrapping_add(4);
    }
}

unsafe fn zero_bss() {
    let mut dst = core::ptr::addr_of_mut!(_sbss) as usize;
    let end = core::ptr::addr_of!(_ebss) as usize;

    while dst < end {
        unsafe {
            core::ptr::write_volatile(dst as *mut u32, 0);
        }
        dst = dst.wrapping_add(4);
    }
}

unsafe fn enable_fpu() {
    const CPACR: *mut u32 = 0xE000_ED88 as *mut u32;
    unsafe {
        core::ptr::write_volatile(CPACR, core::ptr::read_volatile(CPACR) | 0x00F0_0000);
    }
}