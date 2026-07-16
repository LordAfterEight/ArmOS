pub mod kdraw;
pub mod kfont;

pub use kdraw::draw_rect;
pub use kdraw::draw_text;

use alloc::format;
use alloc::string::String;

const CHROME: u32 = 0x00C5_003C;
/// Memory-tracker accent (soft cyan, matches ProcessTracker rows).
const MEM_COLOR: u32 = 0x0055_EAD4;

pub fn kbackground() {
    let fb = &kdraw::GLOBAL_FB
        .get()
        .expect("GLOBAL_FB not initialized")
        .0;
    let base = fb.base();
    let bytes = (fb.pitch * fb.height) as usize;
    unsafe {
        core::ptr::write_bytes(base, 0, bytes);
    }
    draw_rect(
        10,
        55,
        fb.width as u32 - 10 * 2,
        fb.height as u32 - 65,
        2,
        CHROME,
    );
}

/// Human-readable size: B / KiB / MiB with one decimal where useful.
fn format_bytes(n: usize) -> String {
    const KIB: usize = 1024;
    const MIB: usize = 1024 * 1024;
    if n >= MIB {
        let whole = n / MIB;
        let frac = ((n % MIB) * 10) / MIB;
        format!("{whole}.{frac}M")
    } else if n >= KIB {
        let whole = n / KIB;
        let frac = ((n % KIB) * 10) / KIB;
        format!("{whole}.{frac}K")
    } else {
        format!("{n}B")
    }
}

/// Draw free / used / total **SDRAM** (64 MiB main RAM) at the top-right.
pub fn draw_memory_tracker() {
    let s = crate::heap_stats();
    let text = format!(
        "SDRAM  used {}  free {}  total {}",
        format_bytes(s.used),
        format_bytes(s.free),
        format_bytes(s.total),
    );
    let size = 13.0_f32;
    let tw = kdraw::text_length(&text, &kfont::KODEMONO_BOLD, size) as u32;
    let fb = &kdraw::GLOBAL_FB
        .get()
        .expect("GLOBAL_FB not initialized")
        .0;
    let x = fb.width as u32 - tw.saturating_add(16);
    // Align with title band (title uses y≈12, size 28).
    draw_text(x, 18, size, &kfont::KODEMONO_BOLD, &text, MEM_COLOR);
}

/// Paint chrome (background, title, version, memory) into the **back** buffer only.
/// Does not present — callers that also draw content should present once at the end.
pub fn draw_titled_window(title: &str) {
    kbackground();
    // Orbitron title in the chrome band above the content frame (frame starts y=55).
    draw_text(15, 12, 28.0, &kfont::ORBITRON_BOLD, title, CHROME);
    draw_memory_tracker();

    let fb = &kdraw::GLOBAL_FB
        .get()
        .expect("GLOBAL_FB not initialized")
        .0;
    let text = format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let ver_size = 14.0_f32;
    let tw = kdraw::text_length(&text, &kfont::KODEMONO_BOLD, ver_size) as u32;
    let line_h = kdraw::line_height(&kfont::KODEMONO_BOLD, ver_size);
    // Content frame: top y=55, height = fb_h-65 → bottom outer edge at fb_h-10,
    // 2px border → inner floor at fb_h-12. Keep a few px padding above that.
    let ver_y = fb
        .height
        .saturating_sub(12 + 6 + line_h as u64) as u32;
    let ver_x = fb.width as u32 - tw.saturating_add(24);
    draw_text(ver_x, ver_y, ver_size, &kfont::KODEMONO_BOLD, &text, CHROME);
}

/// Full chrome + present (no extra content). Prefer `draw_titled_window` + content +
/// `display::present` when compositing a full frame.
pub fn ktitledwindow(title: &str) {
    draw_titled_window(title);
    crate::drivers::display::present();
}
