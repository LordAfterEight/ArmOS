//! Portable display: double-buffered XRGB8888 framebuffer + scanout backend.
//!
//! Drawing code (`kui`) only touches the back buffer via `GLOBAL_FB`.
//! Profile differences live in `board::{FB_REGION_BASE, SCANOUT}` and the
//! soft / LTDC modules. Future: custom QEMU device can mmap the front buffer.

mod ltdc;
mod soft;

use core::sync::atomic::{AtomicBool, Ordering};

use spin::Mutex;

use crate::board::{self, Scanout};
use crate::kui::kdraw::{Framebuffer, SyncFramebuffer, GLOBAL_FB};

static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Double-buffer indices and which buffer is currently the back (draw target).
struct DisplayState {
    /// 0 or 1 — index of the buffer used for drawing.
    back_idx: u8,
}

static STATE: Mutex<Option<DisplayState>> = Mutex::new(None);

#[inline]
fn buffer_base(index: u8) -> *mut u8 {
    let off = (index as u32 as usize) * (board::FB_BYTES as usize);
    (board::FB_REGION_BASE + off) as *mut u8
}

/// Initialize both framebuffers, install `GLOBAL_FB`, and start scanout.
pub fn init() {
    if INITIALIZED.swap(true, Ordering::SeqCst) {
        return;
    }

    let fb0 = buffer_base(0);
    let fb1 = buffer_base(1);

    // Front starts as 0 (scanned out); drawing goes to back = 1.
    unsafe {
        clear_buffer(fb0);
        clear_buffer(fb1);
    }

    let bases = [fb0, fb1];
    let back_idx = 1u8;

    let fb = Framebuffer {
        bases,
        back_idx: core::sync::atomic::AtomicU8::new(back_idx),
        width: board::FB_WIDTH as u64,
        height: board::FB_HEIGHT as u64,
        pitch: board::FB_STRIDE as u64,
        bpp: board::FB_BPP,
    };

    GLOBAL_FB.call_once(|| SyncFramebuffer(fb));

    *STATE.lock() = Some(DisplayState { back_idx });

    match board::SCANOUT {
        Scanout::Soft => soft::init(buffer_base(0)),
        Scanout::Ltdc => ltdc::init(buffer_base(0)),
    }
}

unsafe fn clear_buffer(base: *mut u8) {
    unsafe {
        core::ptr::write_bytes(base, 0, board::FB_BYTES as usize);
    }
}

/// Swap back/front and notify the scanout backend of the new front buffer.
///
/// After the swap, the new back buffer is **seeded with a copy of the front**
/// so a partial redraw cannot flash a stale/black buffer. Full-frame redraws
/// before `present()` are still the correct UI pattern.
pub fn present() {
    let Some(fb) = GLOBAL_FB.get() else {
        return;
    };

    let mut guard = STATE.lock();
    let Some(state) = guard.as_mut() else {
        return;
    };

    let old_back = state.back_idx;
    let new_back = old_back ^ 1;
    // Old back becomes the new front (now visible).
    let new_front = old_back;

    state.back_idx = new_back;
    fb.0.back_idx.store(new_back, Ordering::Release);

    let front_ptr = buffer_base(new_front);
    let back_ptr = buffer_base(new_back);

    match board::SCANOUT {
        Scanout::Soft => soft::set_front(front_ptr),
        Scanout::Ltdc => ltdc::set_front(front_ptr),
    }

    // Keep both buffers in sync: next draw starts from what is on screen.
    unsafe {
        core::ptr::copy_nonoverlapping(
            front_ptr,
            back_ptr,
            board::FB_BYTES as usize,
        );
    }
}

/// Base address of the buffer currently being scanned out (front).
#[allow(dead_code)]
pub fn front_base() -> Option<*mut u8> {
    let guard = STATE.lock();
    let state = guard.as_ref()?;
    let front = state.back_idx ^ 1;
    Some(buffer_base(front))
}
