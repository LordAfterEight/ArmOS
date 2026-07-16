//! Software scanout — front buffer lives in guest RAM only.
//!
//! No host window yet (QEMU mps2-an500 has no RGB panel model). A future
//! custom QEMU device can poll `FRONT` / guest PSRAM and paint a window.

use core::sync::atomic::{AtomicUsize, Ordering};

/// Physical address of the buffer considered "on screen" (for host tools).
static FRONT: AtomicUsize = AtomicUsize::new(0);

pub fn init(front: *mut u8) {
    FRONT.store(front as usize, Ordering::Release);
}

pub fn set_front(front: *mut u8) {
    FRONT.store(front as usize, Ordering::Release);
}

/// For debuggers / a future QEMU device.
#[inline]
#[allow(dead_code)]
pub fn front_addr() -> usize {
    FRONT.load(Ordering::Acquire)
}
