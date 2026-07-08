pub mod clock;
pub mod gpio;
pub mod pac;
#[cfg(feature = "qemu")]
pub mod semihost;
pub mod uart;