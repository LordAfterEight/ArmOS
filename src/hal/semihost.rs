//! ARM semihosting (used by QEMU with `-semihosting`).

pub fn write_str(s: &str) {
    let mut buf = [0u8; 96];
    let len = s.len().min(buf.len() - 1);
    buf[..len].copy_from_slice(&s.as_bytes()[..len]);

    unsafe {
        core::arch::asm!(
            "mov r1, {ptr}",
            "movs r0, #4",
            "bkpt 0xAB",
            ptr = in(reg) buf.as_ptr() as u32,
            options(nostack, preserves_flags)
        );
    }
}

/// Stop QEMU (or the debugger) after a successful test run.
pub fn exit(code: u32) -> ! {
    let block = [code, 0x20026u32]; // ADP_Stopped_ApplicationExit
    unsafe {
        core::arch::asm!(
            "mov r1, {block}",
            "movs r0, #24",
            "bkpt 0xAB",
            block = in(reg) block.as_ptr() as u32,
            options(nostack, noreturn, preserves_flags)
        );
    }
}