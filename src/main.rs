#![no_std]
#![no_main]

extern crate ArmOS;

mod panic;
mod startup;

#[cfg(feature = "probe-panic")]
use panic_probe as _;

#[unsafe(no_mangle)]
fn main() -> ! {
    ArmOS::drivers::serial::init();
    ArmOS::println!("ArmOS: boot ok");

    let mut scheduler = ArmOS::proc::csched::CooperativeScheduler::init();
    match scheduler.start() {
        Ok(_) => unreachable!("Scheduler only returns on error"),
        Err(e) => ArmOS::println!("Scheduler exited: {e:#?}")
    }

    #[cfg(feature = "qemu")]
    ArmOS::hal::semihost::exit(0);

    #[cfg(not(feature = "qemu"))]
    loop {
        core::hint::spin_loop();
    }
}