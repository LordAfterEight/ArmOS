//! STM32H745 LTDC skeleton for SL-TFT7-class 800×480 parallel RGB.
//!
//! Schematic is not final: pinmux, PLL3 pixel clock, and FMC SDRAM bring-up
//! remain TODOs in `board`. Timings use typical Powertip / SoMLabs values.

use crate::board;
use crate::hal::pac;

pub fn init(front: *mut u8) {
    configure_clocks_stub();
    configure_pins_stub();
    configure_timings_and_layer(front);
    enable_ltdc();
    backlight_stub();
}

pub fn set_front(front: *mut u8) {
    let ltdc = pac::ltdc();
    let layer = ltdc.layer1();
    layer
        .cfbar()
        .write(|w| unsafe { w.cfbadd().bits(front as u32) });
    // Immediate reload (VSYNC-tied reload when tear-free swap is wired).
    ltdc.srcr().write(|w| w.imr().reload());
}

fn configure_clocks_stub() {
    let rcc = pac::rcc();
    // LTDC is on APB3. Pixel clock (typically PLL3 R) is not configured yet —
    // without it the panel will not scan correctly on hardware.
    rcc.apb3enr().modify(|_, w| w.ltdcen().enabled());
    // Dummy read for peripheral bus sync after clock enable.
    let _ = rcc.apb3enr().read().bits();
}

fn configure_pins_stub() {
    // `board::LTDC_PINS` is empty until the carrier schematic freezes.
    let _ = board::LTDC_PINS;
}

fn configure_timings_and_layer(front: *mut u8) {
    let ltdc = pac::ltdc();

    // LTDC timing registers use (N - 1) encoding for widths/accumulated counts.
    // Horizontal:
    //   HSW = hsync - 1
    //   AHBP = hsync + hbp - 1
    //   AAW  = hsync + hbp + active_w - 1
    //   TOTALW = hsync + hbp + active_w + hfp - 1
    // Vertical: same with lines.
    let hsync = board::PANEL_HSYNC as u32;
    let hbp = board::PANEL_HBP as u32;
    let hfp = board::PANEL_HFP as u32;
    let vsync = board::PANEL_VSYNC as u32;
    let vbp = board::PANEL_VBP as u32;
    let vfp = board::PANEL_VFP as u32;
    let aw = board::FB_WIDTH as u32;
    let ah = board::FB_HEIGHT as u32;

    let hsw = hsync.saturating_sub(1);
    let vsh = vsync.saturating_sub(1);
    let ahbp = hsync + hbp - 1;
    let avbp = vsync + vbp - 1;
    let aaw = hsync + hbp + aw - 1;
    let aah = vsync + vbp + ah - 1;
    let totalw = hsync + hbp + aw + hfp - 1;
    let totalh = vsync + vbp + ah + vfp - 1;

    ltdc.sscr().write(|w| unsafe {
        w.hsw().bits(hsw as u16).vsh().bits(vsh as u16)
    });
    ltdc.bpcr().write(|w| unsafe {
        w.ahbp().bits(ahbp as u16).avbp().bits(avbp as u16)
    });
    ltdc.awcr().write(|w| unsafe {
        w.aaw().bits(aaw as u16).aah().bits(aah as u16)
    });
    ltdc.twcr().write(|w| unsafe {
        w.totalw().bits(totalw as u16).totalh().bits(totalh as u16)
    });

    // Background black.
    ltdc.bccr().write(|w| unsafe { w.bcblue().bits(0).bcgreen().bits(0).bcred().bits(0) });

    let layer = ltdc.layer1();

    // Window: full active area (inclusive start/stop in LTDC coords).
    let whstpos = (hsync + hbp) as u16;
    let whsppos = (hsync + hbp + aw - 1) as u16;
    let wvstpos = (vsync + vbp) as u16;
    let wvsppos = (vsync + vbp + ah - 1) as u16;

    layer.whpcr().write(|w| unsafe {
        w.whstpos().bits(whstpos).whsppos().bits(whsppos)
    });
    layer.wvpcr().write(|w| unsafe {
        w.wvstpos().bits(wvstpos).wvsppos().bits(wvsppos)
    });

    // ARGB8888 matches our `u32` `0x00RRGGBB` LE layout (bytes B,G,R,A with A=0).
    layer.pfcr().write(|w| w.pf().argb8888());

    // Constant alpha 0xFF, no blending tricks for single-layer full-screen.
    layer.cacr().write(|w| unsafe { w.consta().bits(0xFF) });
    layer.bfcr().write(|w| w.bf1().constant().bf2().constant());

    layer
        .cfbar()
        .write(|w| unsafe { w.cfbadd().bits(front as u32) });

    // CFBLL = line length in bytes + 3; CFBP = pitch in bytes.
    let pitch = board::FB_STRIDE;
    let line_len = pitch + 3;
    layer.cfblr().write(|w| unsafe {
        w.cfbll().bits(line_len as u16).cfbp().bits(pitch as u16)
    });
    layer
        .cfblnr()
        .write(|w| unsafe { w.cfblnbr().bits(board::FB_HEIGHT as u16) });

    // Enable layer 1.
    layer.cr().modify(|_, w| w.len().enabled());

    // Reload shadow registers immediately.
    ltdc.srcr().write(|w| w.imr().reload());
}

fn enable_ltdc() {
    let ltdc = pac::ltdc();
    ltdc.gcr().modify(|_, w| w.ltdcen().enabled());
}

fn backlight_stub() {
    // board::BACKLIGHT_PWREN / BACKLIGHT_PWM — drive when pins are assigned.
    let _ = (board::BACKLIGHT_PWREN, board::BACKLIGHT_PWM);
}
