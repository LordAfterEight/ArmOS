//! Font assets — fontdue faces embedded from `assets/fonts/`.
//!
//! Only fonts required for boot chrome / ProcessTracker are loaded in `init`
//! to stay within the bump-heap budget. Others load on first use via `ensure_*`.

use spin::Once;

pub static KFONT: Once<fontdue::Font> = Once::new();
pub static NABLA_REGULAR: Once<fontdue::Font> = Once::new();
pub static ORBITRON_LIGHT: Once<fontdue::Font> = Once::new();
pub static ORBITRON_REGULAR: Once<fontdue::Font> = Once::new();
pub static ORBITRON_BOLD: Once<fontdue::Font> = Once::new();
pub static KODEMONO_REGULAR: Once<fontdue::Font> = Once::new();
pub static KODEMONO_BOLD: Once<fontdue::Font> = Once::new();
pub static ICELAND: Once<fontdue::Font> = Once::new();
pub static BARCODE: Once<fontdue::Font> = Once::new();

static KFONT_BYTES_REGULAR: &[u8] = include_bytes!("../../assets/fonts/Datatype-Regular.ttf");
static ORBITRON_BYTES_LIGHT: &[u8] = include_bytes!("../../assets/fonts/Orbitron Light.ttf");
static ORBITRON_BYTES_REGULAR: &[u8] = include_bytes!("../../assets/fonts/Orbitron Regular.ttf");
static ORBITRON_BYTES_BOLD: &[u8] = include_bytes!("../../assets/fonts/Orbitron Bold.ttf");
static KODEMONO_BYTES_REGULAR: &[u8] = include_bytes!("../../assets/fonts/KodeMono-Regular.ttf");
static KODEMONO_BYTES_BOLD: &[u8] = include_bytes!("../../assets/fonts/KodeMono-Bold.ttf");
static NABLA_BYTES_REGULAR: &[u8] = include_bytes!("../../assets/fonts/Nabla-Regular.ttf");
static ICELAND_BYTES: &[u8] = include_bytes!("../../assets/fonts/Iceland-Regular.ttf");
static BARCODE_BYTES: &[u8] =
    include_bytes!("../../assets/fonts/LibreBarcode128Text-Regular.ttf");

fn load(bytes: &[u8]) -> fontdue::Font {
    fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())
        .expect("font parse failed")
}

/// Load fonts needed for early UI (chrome + ProcessTracker).
pub fn init() {
    // Readable UI faces first. Barcode is optional accent / version tag only.
    KODEMONO_BOLD.call_once(|| load(KODEMONO_BYTES_BOLD));
    ORBITRON_BOLD.call_once(|| load(ORBITRON_BYTES_BOLD));
    BARCODE.call_once(|| load(BARCODE_BYTES));
}

#[allow(dead_code)]
pub fn ensure_kfont() {
    KFONT.call_once(|| load(KFONT_BYTES_REGULAR));
}

#[allow(dead_code)]
pub fn ensure_orbitron() {
    ORBITRON_LIGHT.call_once(|| load(ORBITRON_BYTES_LIGHT));
    ORBITRON_REGULAR.call_once(|| load(ORBITRON_BYTES_REGULAR));
    ORBITRON_BOLD.call_once(|| load(ORBITRON_BYTES_BOLD));
}

#[allow(dead_code)]
pub fn ensure_kodemono_regular() {
    KODEMONO_REGULAR.call_once(|| load(KODEMONO_BYTES_REGULAR));
}

#[allow(dead_code)]
pub fn ensure_iceland() {
    ICELAND.call_once(|| load(ICELAND_BYTES));
}

#[allow(dead_code)]
pub fn ensure_nabla() {
    // Color font — may fail under fontdue; ignore if so.
    if NABLA_REGULAR.get().is_some() {
        return;
    }
    if let Ok(font) =
        fontdue::Font::from_bytes(NABLA_BYTES_REGULAR, fontdue::FontSettings::default())
    {
        NABLA_REGULAR.call_once(|| font);
    }
}
