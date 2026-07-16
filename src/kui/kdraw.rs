//! Framebuffer drawing — portable XRGB8888 back-buffer ops (no board cfg).

use core::sync::atomic::{AtomicU8, Ordering};

use spin::Once;

/// Double-buffered framebuffer. Drawing always targets `base()` (back buffer).
pub struct Framebuffer {
    pub bases: [*mut u8; 2],
    /// Index of the buffer currently used for drawing (0 or 1).
    pub back_idx: AtomicU8,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16,
}

// SAFETY: bases point at reserved display RAM; access is coordinated by the
// display driver (single-core for now).
unsafe impl Sync for Framebuffer {}
unsafe impl Send for Framebuffer {}

impl Framebuffer {
    #[inline]
    pub fn base(&self) -> *mut u8 {
        let i = self.back_idx.load(Ordering::Acquire) as usize & 1;
        self.bases[i]
    }

    #[inline]
    pub fn bytes_per_pixel(&self) -> usize {
        (self.bpp as usize) / 8
    }
}

pub struct SyncFramebuffer(pub Framebuffer);

unsafe impl Sync for SyncFramebuffer {}
unsafe impl Send for SyncFramebuffer {}

pub static GLOBAL_FB: Once<SyncFramebuffer> = Once::new();

#[inline]
fn fb() -> &'static Framebuffer {
    &GLOBAL_FB
        .get()
        .expect("GLOBAL_FB not initialized — call display::init() first")
        .0
}

/// Store `color` as little-endian XRGB8888 (`0x00RRGGBB` → bytes B,G,R,0).
#[inline]
pub fn put_pixel(x: u32, y: u32, color: u32) {
    let fb = fb();
    if (x as u64) >= fb.width || (y as u64) >= fb.height {
        return;
    }
    let bpp = fb.bytes_per_pixel();
    if bpp != 4 {
        return;
    }
    let offset = (y as u64 * fb.pitch) + (x as u64 * 4);
    unsafe {
        let ptr = fb.base().add(offset as usize) as *mut u32;
        core::ptr::write_volatile(ptr, color);
    }
}

#[inline]
fn get_pixel(x: u32, y: u32) -> u32 {
    let fb = fb();
    if (x as u64) >= fb.width || (y as u64) >= fb.height {
        return 0;
    }
    let offset = (y as u64 * fb.pitch) + (x as u64 * 4);
    unsafe {
        let ptr = fb.base().add(offset as usize) as *const u32;
        core::ptr::read_volatile(ptr)
    }
}

/// Blend `src` over `dst` with 8-bit coverage (0 = dst, 255 = src).
#[inline]
fn blend_coverage(dst: u32, src: u32, cover: u8) -> u32 {
    if cover == 0 {
        return dst;
    }
    if cover == 255 {
        return src;
    }
    let a = cover as u32;
    let inv = 255 - a;
    let db = dst & 0xff;
    let dg = (dst >> 8) & 0xff;
    let dr = (dst >> 16) & 0xff;
    let sb = src & 0xff;
    let sg = (src >> 8) & 0xff;
    let sr = (src >> 16) & 0xff;
    let b = (sb * a + db * inv) / 255;
    let g = (sg * a + dg * inv) / 255;
    let r = (sr * a + dr * inv) / 255;
    (r << 16) | (g << 8) | b
}

pub fn text_length(text: &str, font: &Once<fontdue::Font>, size: f32) -> usize {
    let Some(font) = font.get() else {
        return 0;
    };
    let mut width = 0.0f32;
    for ch in text.chars() {
        let metrics = font.metrics(ch, size);
        width += metrics.advance_width;
    }
    width as usize
}

#[inline]
fn ceil_u32(v: f32) -> u32 {
    let t = v as u32;
    if v > t as f32 {
        t + 1
    } else {
        t.max(1)
    }
}

/// Approximate line box height for layout (ascent − descent + gap).
pub fn line_height(font: &Once<fontdue::Font>, size: f32) -> u32 {
    let Some(font) = font.get() else {
        return ceil_u32(size);
    };
    font.horizontal_line_metrics(size)
        .map(|m| ceil_u32(m.new_line_size.max(1.0)))
        .unwrap_or(ceil_u32(size))
}

/// Draw text with the **top** of the line box at `(x, y)` (y grows downward).
///
/// Fontdue `Metrics::ymin` is the offset of the bitmap's **bottom** edge from
/// the baseline in y-up font space (often ≤ 0). For a y-down framebuffer the
/// top of the glyph bitmap is:
/// `baseline - ymin - height`.
pub fn draw_text(
    x: u32,
    y: u32,
    size: f32,
    font: &Once<fontdue::Font>,
    text: &str,
    color: u32,
) {
    let Some(font) = font.get() else {
        return;
    };

    // Top of line box → baseline (ascent is distance above baseline, ≥ 0).
    let ascent = font
        .horizontal_line_metrics(size)
        .map(|m| m.ascent)
        .unwrap_or(size * 0.8);
    let baseline = y as f32 + ascent;

    let mut pen_x = x as f32;

    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, size);
        // xmin: left of bitmap relative to pen (may be negative).
        // ymin: bottom of bitmap relative to baseline in y-up coords.
        let glyph_x = pen_x + metrics.xmin as f32;
        let glyph_y = baseline - metrics.ymin as f32 - metrics.height as f32;

        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let coverage = bitmap[row * metrics.width + col];
                if coverage == 0 {
                    continue;
                }
                // no_std: truncating cast is fine for pixel centers
                let px = (glyph_x + col as f32) as i32;
                let py = (glyph_y + row as f32) as i32;
                if px < 0 || py < 0 {
                    continue;
                }
                let px = px as u32;
                let py = py as u32;
                if coverage >= 250 {
                    put_pixel(px, py, color);
                } else {
                    let dst = get_pixel(px, py);
                    put_pixel(px, py, blend_coverage(dst, color, coverage));
                }
            }
        }
        pen_x += metrics.advance_width;
    }
}
/// Draw a rectangle. `t == 0` fills solid; otherwise draws a border of thickness `t`.
pub fn draw_rect(x: u32, y: u32, w: u32, h: u32, t: u16, col: u32) {
    if w == 0 || h == 0 {
        return;
    }

    if t == 0 {
        for row in y..y.saturating_add(h) {
            for col_x in x..x.saturating_add(w) {
                put_pixel(col_x, row, col);
            }
        }
        return;
    }

    let t = t as u32;
    // Top
    for row in y..y.saturating_add(t).min(y.saturating_add(h)) {
        for col_x in x..x.saturating_add(w) {
            put_pixel(col_x, row, col);
        }
    }
    // Bottom
    if h > t {
        let y1 = y + h - t;
        for row in y1..y.saturating_add(h) {
            for col_x in x..x.saturating_add(w) {
                put_pixel(col_x, row, col);
            }
        }
    }
    // Sides
    for row in y..y.saturating_add(h) {
        for col_x in x..x.saturating_add(t).min(x.saturating_add(w)) {
            put_pixel(col_x, row, col);
        }
        if w > t {
            let x1 = x + w - t;
            for col_x in x1..x.saturating_add(w) {
                put_pixel(col_x, row, col);
            }
        }
    }
}
