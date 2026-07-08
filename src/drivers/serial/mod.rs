//! Serial console — USART1 on hardware, semihosting under QEMU.
//!
//! x86 kernels often use `COM1` (`0x3F8`); on this STM32H745 target the
//! boot console is USART1.

use core::fmt::Write;

use crate::hal::uart;

/// USART1 peripheral base (STM32H745). Role-equivalent to x86 `COM1`.
pub const PORT: usize = 0x4001_1000;

pub struct Serial;

pub fn init() {
    uart::init();
}

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            uart::write_byte(byte);
        }
        Ok(())
    }
}

/// Prints a formatted string to the serial console.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = core::write!($crate::drivers::serial::Serial, $($arg)*);
    }};
}

/// Prints a formatted string to the serial console, followed by a newline.
#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = core::writeln!($crate::drivers::serial::Serial, $($arg)*);
    }};
}