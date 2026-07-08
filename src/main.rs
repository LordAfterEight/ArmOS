#![no_std]
#![no_main]

mod board;
mod hal;
mod panic;
mod startup;

use hal::uart;

#[cfg(feature = "probe-panic")]
use panic_probe as _;

#[unsafe(no_mangle)]
fn main() -> ! {
    uart::init();
    uart::write_str("ArmOS: boot ok\r\n");

    #[cfg(feature = "qemu")]
    crate::hal::semihost::exit(0);

    #[cfg(not(feature = "qemu"))]
    loop {
        core::hint::spin_loop();
    }
}