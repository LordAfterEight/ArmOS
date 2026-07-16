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

    ArmOS::drivers::display::init();
    ArmOS::kui::kfont::init();
    ArmOS::println!("ArmOS: display ready");

    let mut scheduler = ArmOS::proc::csched::CooperativeScheduler::init();
    scheduler.add_process::<ArmOS::apps::proctracker::ProcessTracker>();
    match scheduler.start() {
        Ok(_) => unreachable!("Scheduler only returns on error"),
        Err(e) => {
            ArmOS::println!("Scheduler exited: {e:#?}");
        }
    }

    loop {
        core::hint::spin_loop();
    }
}
