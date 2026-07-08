//! Framebuffer drawing — stubbed until display abstractions exist.

use spin::Once;

/// Placeholder framebuffer type (replaces OwOS/limine `Framebuffer` for now).
pub struct Framebuffer {
    pub base: *mut u8,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16,
}

pub struct SyncFramebuffer(pub &'static Framebuffer);

unsafe impl Sync for SyncFramebuffer {}
unsafe impl Send for SyncFramebuffer {}

pub static GLOBAL_FB: Once<SyncFramebuffer> = Once::new();

pub fn text_length(_text: &str, _font: &Once<fontdue::Font>, _size: f32) -> usize {
    // Stub: no text metrics until font/display abstractions are wired up.
    0
}

pub fn draw_text(
    _x: u32,
    _y: u32,
    _size: f32,
    _font: &Once<fontdue::Font>,
    _text: &str,
    _color: u32,
) {
    // Note: no-op until display abstraction exists.
}

pub fn draw_rect(_x: u32, _y: u32, _w: u32, _h: u32, _t: u16, _col: u32) {
    // Note: no-op until display abstraction exists.
}

// fn fb() -> &'static Framebuffer {
//     GLOBAL_FB.get().expect("GLOBAL_FB not initialized").0
// }
//
// fn draw_glyph(...) { ... }
//
// pub fn text_length(text: &str, font: &Once<fontdue::Font>, size: f32) -> usize {
//     let mut width = 0;
//     for char in text.chars() {
//         let (metrics, _bitmap) = font.get().unwrap().rasterize(char, size);
//         width += metrics.advance_width as usize;
//     }
//     width
// }
//
// pub fn draw_text(x, y, size, font, text, color) { ... }
//
// pub fn draw_rect(x, y, w, h, t, col) { ... }