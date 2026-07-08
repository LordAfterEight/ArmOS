#[cfg(not(feature = "probe-panic"))]
use core::arch::asm;
#[cfg(not(feature = "probe-panic"))]
use core::panic::PanicInfo;

#[cfg(not(feature = "probe-panic"))]
use crate::hal::uart;

#[cfg(not(feature = "probe-panic"))]
const MAX_BACKTRACE_FRAMES: usize = 16;

#[cfg(not(feature = "probe-panic"))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    uart::write_str("PANIC");

    if let Some(loc) = info.location() {
        uart::write_fmt(format_args!(" at {}:{}", loc.file(), loc.line()));
    }

    uart::write_fmt(format_args!(": {}", info.message()));
    uart::write_str("\r\n");

    let mut pc: u32;
    let mut lr: u32;
    unsafe {
        asm!("mov {}, pc", out(reg) pc, options(nomem, nostack));
        asm!("mov {}, lr", out(reg) lr, options(nomem, nostack));
    }
    uart::write_fmt(format_args!("pc=0x{pc:08x} lr=0x{lr:08x}\r\n"));

    print_backtrace();

    loop {}
}

#[cfg(not(feature = "probe-panic"))]
fn print_backtrace() {
    let mut fp: usize;
    unsafe {
        asm!("mov {}, r7", out(reg) fp, options(nomem, nostack));
    }

    for i in 0..MAX_BACKTRACE_FRAMES {
        if fp == 0 || !fp_in_dtcm(fp) {
            break;
        }

        let lr = unsafe { core::ptr::read_volatile((fp + 4) as *const u32) };
        uart::write_fmt(format_args!("bt[{i}]=0x{lr:08x}\r\n"));

        let next_fp = unsafe { core::ptr::read_volatile(fp as *const u32) as usize };
        if next_fp <= fp {
            break;
        }
        fp = next_fp;
    }
}

#[cfg(not(feature = "probe-panic"))]
fn fp_in_dtcm(fp: usize) -> bool {
    fp >= 0x2000_0000 && fp < 0x2002_0000
}