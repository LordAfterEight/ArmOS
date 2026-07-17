//! Stage-0 bootloader: MCU flash + DTCM only.
//!
//! **Same sequence on silicon and on `stm32h745-carrier` QEMU** — no shortcuts.
//! QEMU will not map SDRAM/NOR until every step below is done.
//!
//! 1. Enable GPIO clocks (RCC AHB4ENR) for ports used by FMC/QSPI
//! 2. Pinmux FMC SDRAM bus (AF12) + QSPI (AF9/10) + NOR RESET# (PI11 high)
//!    including **PG6 = QSPI NCS** (board bodge of schematic `QSPI_NCS`)
//! 3. Enable FMC + QUADSPI clocks (RCC AHB3ENR)
//! 4. FMC SDRAM bank1 init: CLOCK → PALL → AUTOREF → LOAD_MODE
//! 5. QUADSPI memory-mapped mode (DCR.FSIZE, CCR.FMODE=11, CR.EN)
//! 6. Validate NOR vectors; if bad → ERR LED + LTDC fail UI (text), never jump
//! 7. Else RUN LED on, jump to ArmOS @ 0x90000000
//!
//! Board: STM32H745BITx + IS25LP01GJ (QSPI) + AS4C32M16SB (FMC SDRAM).
//! Status LEDs (anode ← GPIO, active high): PE2=RUN PE3=DBG PE4=ERR PE5=PNC.
//! Fail panel: SDRAM FB @ 0xC0000000 + LTDC layer 1 (tiny 5×7 font, no OS deps).

#![no_std]
#![no_main]

const OS_VECTOR_TABLE: u32 = 0x9000_0000;

/* ── RCC ─────────────────────────────────────────────────────────── */
const RCC_AHB3ENR: *mut u32 = 0x5802_44D4 as *mut u32;
const RCC_AHB4ENR: *mut u32 = 0x5802_44E0 as *mut u32;
const RCC_AHB3ENR_FMCEN: u32 = 1 << 12;
const RCC_AHB3ENR_QSPIEN: u32 = 1 << 14;
const RCC_AHB4ENR_GPIOCEN: u32 = 1 << 2;
const RCC_AHB4ENR_GPIODEN: u32 = 1 << 3;
const RCC_AHB4ENR_GPIOEEN: u32 = 1 << 4;
const RCC_AHB4ENR_GPIOFEN: u32 = 1 << 5;
const RCC_AHB4ENR_GPIOGEN: u32 = 1 << 6;
const RCC_AHB4ENR_GPIOHEN: u32 = 1 << 7;
const RCC_AHB4ENR_GPIOIEN: u32 = 1 << 8;

/* ── GPIO bases ──────────────────────────────────────────────────── */
const GPIOC: usize = 0x5802_0800;
const GPIOD: usize = 0x5802_0C00;
const GPIOE: usize = 0x5802_1000;
const GPIOF: usize = 0x5802_1400;
const GPIOG: usize = 0x5802_1800;
const GPIOH: usize = 0x5802_1C00;
const GPIOI: usize = 0x5802_2000;

const GPIO_MODER: usize = 0x00;
const GPIO_OSPEEDR: usize = 0x08;
const GPIO_ODR: usize = 0x14;
const GPIO_BSRR: usize = 0x18;
const GPIO_AFRL: usize = 0x20;
const GPIO_AFRH: usize = 0x24;

const MODE_OUTPUT: u32 = 0b01;
const MODE_AF: u32 = 0b10;
const SPEED_VERY_HIGH: u32 = 0b11;
const AF_FMC: u32 = 12;
const AF_QSPI_9: u32 = 9;
const AF_QSPI_10: u32 = 10;

/* ── FMC SDRAM ───────────────────────────────────────────────────── */
const FMC_SDCR1: *mut u32 = 0x5200_4140 as *mut u32;
const FMC_SDTR1: *mut u32 = 0x5200_4148 as *mut u32;
const FMC_SDCMR: *mut u32 = 0x5200_4150 as *mut u32;
const FMC_SDRTR: *mut u32 = 0x5200_4154 as *mut u32;
const FMC_SDSR: *const u32 = 0x5200_4158 as *const u32;
const FMC_SDCMR_CTB1: u32 = 1 << 4;

/* ── QUADSPI ─────────────────────────────────────────────────────── */
const QUADSPI_CR: *mut u32 = 0x5200_5000 as *mut u32;
const QUADSPI_DCR: *mut u32 = 0x5200_5004 as *mut u32;
const QUADSPI_CCR: *mut u32 = 0x5200_5014 as *mut u32;
const QUADSPI_CR_EN: u32 = 1 << 0;
const QUADSPI_CCR_FMODE_MEMMAP: u32 = 3 << 26;

/* ── Status LEDs (carrier D4–D7) ─────────────────────────────────── */
const LED_RUN: u32 = 2; // PE2
const LED_DBG: u32 = 3; // PE3
const LED_ERR: u32 = 4; // PE4
const LED_PNC: u32 = 5; // PE5

/* ── LTDC (fail screen only; OS owns full bring-up later) ────────── */
const RCC_APB3ENR: *mut u32 = 0x5802_44E4 as *mut u32;
const RCC_APB3ENR_LTDCEN: u32 = 1 << 3;
const LTDC_BASE: usize = 0x5000_1000;
const FB_BASE: u32 = 0xC000_0000;
const FB_W: u32 = 800;
const FB_H: u32 = 480;
const FB_STRIDE: u32 = FB_W * 4;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    // Best-effort: PE5 PANIC high if GPIOE is already clocked.
    unsafe {
        gpio_set_output_level(GPIOE, LED_PNC, true);
    }
    loop {
        core::hint::spin_loop();
    }
}

core::arch::global_asm!(
    r#"
    .section .vector_table, "a"
    .word _estack
    .word Reset
    .word DefaultHandler
    .word DefaultHandler
    .word DefaultHandler
    .word DefaultHandler
    .word DefaultHandler
    .word 0
    .word 0
    .word 0
    .word 0
    .word DefaultHandler
    .word DefaultHandler
    .word 0
    .word DefaultHandler
    .word DefaultHandler
    .rept 149
    .word DefaultHandler
    .endr
    "#,
);

#[unsafe(no_mangle)]
pub extern "C" fn DefaultHandler() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

unsafe extern "C" {
    static _sidata: u32;
    static mut _sdata: u32;
    static _edata: u32;
    static mut _sbss: u32;
    static _ebss: u32;
}

#[inline(always)]
unsafe fn reg_write(addr: *mut u32, val: u32) {
    unsafe { core::ptr::write_volatile(addr, val) }
}

#[inline(always)]
unsafe fn reg_read(addr: *const u32) -> u32 {
    unsafe { core::ptr::read_volatile(addr) }
}

#[inline(always)]
unsafe fn gpio_reg(port: usize, off: usize) -> *mut u32 {
    (port + off) as *mut u32
}

/// Set pin mode (2 bits in MODER) and optional AFR / speed.
unsafe fn gpio_set_pin(port: usize, pin: u32, mode: u32, af: u32) {
    unsafe {
        let moder = gpio_reg(port, GPIO_MODER);
        let m = reg_read(moder);
        reg_write(moder, (m & !(0b11 << (pin * 2))) | (mode << (pin * 2)));

        let speed = gpio_reg(port, GPIO_OSPEEDR);
        let sp = reg_read(speed);
        reg_write(
            speed,
            (sp & !(0b11 << (pin * 2))) | (SPEED_VERY_HIGH << (pin * 2)),
        );

        if mode == MODE_AF {
            if pin < 8 {
                let afr = gpio_reg(port, GPIO_AFRL);
                let a = reg_read(afr);
                reg_write(afr, (a & !(0xF << (pin * 4))) | (af << (pin * 4)));
            } else {
                let afr = gpio_reg(port, GPIO_AFRH);
                let shift = (pin - 8) * 4;
                let a = reg_read(afr);
                reg_write(afr, (a & !(0xF << shift)) | (af << shift));
            }
        }
    }
}

unsafe fn gpio_set_output_level(port: usize, pin: u32, high: bool) {
    unsafe {
        gpio_set_pin(port, pin, MODE_OUTPUT, 0);
        if high {
            reg_write(gpio_reg(port, GPIO_BSRR), 1 << pin);
        } else {
            reg_write(gpio_reg(port, GPIO_BSRR), 1 << (pin + 16));
        }
        let _ = reg_read(gpio_reg(port, GPIO_ODR));
    }
}

unsafe fn gpio_set_output_high(port: usize, pin: u32) {
    unsafe {
        gpio_set_output_level(port, pin, true);
    }
}

/// Carrier status LEDs — PE2 RUN, PE3 DBG, PE4 ERR, PE5 PNC (active high).
unsafe fn status_leds_init() {
    unsafe {
        for pin in [LED_RUN, LED_DBG, LED_ERR, LED_PNC] {
            gpio_set_output_level(GPIOE, pin, false);
        }
    }
}

unsafe fn led_set(pin: u32, on: bool) {
    unsafe {
        gpio_set_output_level(GPIOE, pin, on);
    }
}

/// Carrier pinmux — must match QEMU `carrier_*_pinmux_ok` checks.
unsafe fn board_pinmux_init() {
    unsafe {
        /* ── QSPI: IS25LP01GJ ─────────────────────────────────────── */
        gpio_set_pin(GPIOF, 6, MODE_AF, AF_QSPI_9); // IO3
        gpio_set_pin(GPIOF, 7, MODE_AF, AF_QSPI_9); // IO2
        gpio_set_pin(GPIOF, 8, MODE_AF, AF_QSPI_10); // IO0
        gpio_set_pin(GPIOF, 9, MODE_AF, AF_QSPI_10); // IO1
        gpio_set_pin(GPIOF, 10, MODE_AF, AF_QSPI_9); // CLK
        gpio_set_pin(GPIOG, 6, MODE_AF, AF_QSPI_10); // NCS (bodge)
        gpio_set_output_high(GPIOI, 11); // NOR RESET# released

        /* ── FMC SDRAM: AS4C32M16SB ────────────────────────────────── */
        for pin in 0u32..=5 {
            gpio_set_pin(GPIOF, pin, MODE_AF, AF_FMC); // A0–A5
        }
        gpio_set_pin(GPIOF, 11, MODE_AF, AF_FMC); // RAS
        for pin in 12u32..=15 {
            gpio_set_pin(GPIOF, pin, MODE_AF, AF_FMC); // A6–A9
        }
        gpio_set_pin(GPIOG, 0, MODE_AF, AF_FMC); // A10
        gpio_set_pin(GPIOG, 1, MODE_AF, AF_FMC); // A11
        gpio_set_pin(GPIOG, 2, MODE_AF, AF_FMC); // A12
        gpio_set_pin(GPIOG, 4, MODE_AF, AF_FMC); // BA0
        gpio_set_pin(GPIOG, 5, MODE_AF, AF_FMC); // BA1
        gpio_set_pin(GPIOG, 8, MODE_AF, AF_FMC); // SDCLK
        gpio_set_pin(GPIOG, 15, MODE_AF, AF_FMC); // CAS
        gpio_set_pin(GPIOC, 0, MODE_AF, AF_FMC); // WE
        gpio_set_pin(GPIOC, 2, MODE_AF, AF_FMC); // SDNE0 / CS
        gpio_set_pin(GPIOH, 7, MODE_AF, AF_FMC); // CKE
        gpio_set_pin(GPIOE, 0, MODE_AF, AF_FMC); // NBL0
        gpio_set_pin(GPIOE, 1, MODE_AF, AF_FMC); // NBL1
        gpio_set_pin(GPIOD, 14, MODE_AF, AF_FMC); // D0
        gpio_set_pin(GPIOD, 15, MODE_AF, AF_FMC); // D1
        gpio_set_pin(GPIOD, 0, MODE_AF, AF_FMC); // D2
        gpio_set_pin(GPIOD, 1, MODE_AF, AF_FMC); // D3
        for pin in 7u32..=15 {
            gpio_set_pin(GPIOE, pin, MODE_AF, AF_FMC); // D4–D12
        }
        gpio_set_pin(GPIOD, 8, MODE_AF, AF_FMC); // D13
        gpio_set_pin(GPIOD, 9, MODE_AF, AF_FMC); // D14
        gpio_set_pin(GPIOD, 10, MODE_AF, AF_FMC); // D15
    }
}

unsafe fn fmc_wait_ready() {
    unsafe {
        while reg_read(FMC_SDSR) & 1 != 0 {}
    }
}

/// FMC SDRAM bank1 init — RM0399 command order.
unsafe fn fmc_sdram_init() {
    unsafe {
        // SDCR1: 16-bit, CAS2, 4 banks, 13 row / 9 col (match AS4C32M16 class)
        reg_write(FMC_SDCR1, 0x0000_1959);
        reg_write(FMC_SDTR1, 0x0111_5361);
        fmc_wait_ready();

        reg_write(FMC_SDCMR, FMC_SDCMR_CTB1 | 1); // CLOCK
        fmc_wait_ready();
        reg_write(FMC_SDCMR, FMC_SDCMR_CTB1 | 2); // PALL
        fmc_wait_ready();
        reg_write(FMC_SDCMR, FMC_SDCMR_CTB1 | (1 << 5) | 3); // AUTOREF
        fmc_wait_ready();
        reg_write(FMC_SDCMR, FMC_SDCMR_CTB1 | (0x220 << 9) | 4); // LOAD
        fmc_wait_ready();

        reg_write(FMC_SDRTR, 0x27C << 1);
        fmc_wait_ready();
    }
}

/// QUADSPI memory-mapped XIP for 128 MiB IS25LP01GJ (FSIZE=26).
unsafe fn quadspi_enable_memmap() {
    unsafe {
        reg_write(QUADSPI_DCR, 26 << 16);
        reg_write(QUADSPI_CCR, QUADSPI_CCR_FMODE_MEMMAP);
        reg_write(QUADSPI_CR, QUADSPI_CR_EN);
    }
}

#[inline(always)]
unsafe fn ltdc_w(off: usize, val: u32) {
    unsafe {
        reg_write((LTDC_BASE + off) as *mut u32, val);
    }
}

/* ── Fail-panel drawing (SDRAM FB; no heap / no OS fonts) ────────── */

const COL_BLACK: u32 = 0x0000_0000;
const COL_RED: u32 = 0x00FF_0000;

/// 5×7 glyphs, column-major, bit0 = top. Index = ASCII - 0x20 for 0x20..=0x5F.
/// Missing glyphs render as a solid block.
#[rustfmt::skip]
const FONT5X7: [[u8; 5]; 64] = [
    // sp ! " # $ % & ' ( ) * + , - . /
    [0x00,0x00,0x00,0x00,0x00], [0x00,0x00,0x5F,0x00,0x00], [0x00,0x07,0x00,0x07,0x00],
    [0x14,0x7F,0x14,0x7F,0x14], [0x24,0x2A,0x7F,0x2A,0x12], [0x23,0x13,0x08,0x64,0x62],
    [0x36,0x49,0x55,0x22,0x50], [0x00,0x05,0x03,0x00,0x00], [0x00,0x1C,0x22,0x41,0x00],
    [0x00,0x41,0x22,0x1C,0x00], [0x14,0x08,0x3E,0x08,0x14], [0x08,0x08,0x3E,0x08,0x08],
    [0x00,0x50,0x30,0x00,0x00], [0x08,0x08,0x08,0x08,0x08], [0x00,0x60,0x60,0x00,0x00],
    [0x20,0x10,0x08,0x04,0x02],
    // 0-9
    [0x3E,0x51,0x49,0x45,0x3E], [0x00,0x42,0x7F,0x40,0x00], [0x42,0x61,0x51,0x49,0x46],
    [0x21,0x41,0x45,0x4B,0x31], [0x18,0x14,0x12,0x7F,0x10], [0x27,0x45,0x45,0x45,0x39],
    [0x3C,0x4A,0x49,0x49,0x30], [0x01,0x71,0x09,0x05,0x03], [0x36,0x49,0x49,0x49,0x36],
    [0x06,0x49,0x49,0x29,0x1E],
    // : ; < = > ? @
    [0x00,0x36,0x36,0x00,0x00], [0x00,0x56,0x36,0x00,0x00], [0x08,0x14,0x22,0x41,0x00],
    [0x14,0x14,0x14,0x14,0x14], [0x00,0x41,0x22,0x14,0x08], [0x02,0x01,0x51,0x09,0x06],
    [0x32,0x49,0x79,0x41,0x3E],
    // A-Z
    [0x7E,0x11,0x11,0x11,0x7E], [0x7F,0x49,0x49,0x49,0x36], [0x3E,0x41,0x41,0x41,0x22],
    [0x7F,0x41,0x41,0x22,0x1C], [0x7F,0x49,0x49,0x49,0x41], [0x7F,0x09,0x09,0x09,0x01],
    [0x3E,0x41,0x49,0x49,0x7A], [0x7F,0x08,0x08,0x08,0x7F], [0x00,0x41,0x7F,0x41,0x00],
    [0x20,0x40,0x41,0x3F,0x01], [0x7F,0x08,0x14,0x22,0x41], [0x7F,0x40,0x40,0x40,0x40],
    [0x7F,0x02,0x0C,0x02,0x7F], [0x7F,0x04,0x08,0x10,0x7F], [0x3E,0x41,0x41,0x41,0x3E],
    [0x7F,0x09,0x09,0x09,0x06], [0x3E,0x41,0x51,0x21,0x5E], [0x7F,0x09,0x19,0x29,0x46],
    [0x46,0x49,0x49,0x49,0x31], [0x01,0x01,0x7F,0x01,0x01], [0x3F,0x40,0x40,0x40,0x3F],
    [0x1F,0x20,0x40,0x20,0x1F], [0x3F,0x40,0x38,0x40,0x3F], [0x63,0x14,0x08,0x14,0x63],
    [0x07,0x08,0x70,0x08,0x07], [0x61,0x51,0x49,0x45,0x43],
    // [ \ ] ^ _ 
    [0x00,0x7F,0x41,0x41,0x00], [0x02,0x04,0x08,0x10,0x20], [0x00,0x41,0x41,0x7F,0x00],
    [0x04,0x02,0x01,0x02,0x04], [0x40,0x40,0x40,0x40,0x40],
];

#[inline(always)]
unsafe fn fb_put(x: u32, y: u32, color: u32) {
    if x < FB_W && y < FB_H {
        let i = (y * FB_W + x) as usize;
        unsafe {
            core::ptr::write_volatile((FB_BASE as *mut u32).add(i), color);
        }
    }
}

unsafe fn fb_fill(x: u32, y: u32, w: u32, h: u32, color: u32) {
    let x1 = x.saturating_add(w).min(FB_W);
    let y1 = y.saturating_add(h).min(FB_H);
    let mut yy = y.min(FB_H);
    while yy < y1 {
        let mut xx = x.min(FB_W);
        while xx < x1 {
            unsafe { fb_put(xx, yy, color) };
            xx += 1;
        }
        yy += 1;
    }
}

unsafe fn fb_fill_screen(color: u32) {
    unsafe {
        fb_fill(0, 0, FB_W, FB_H, color);
    }
}

fn glyph5x7(c: u8) -> [u8; 5] {
    let u = if (b'a'..=b'z').contains(&c) {
        c - (b'a' - b'A')
    } else {
        c
    };
    if (0x20..=0x5F).contains(&u) {
        FONT5X7[(u - 0x20) as usize]
    } else {
        [0x7F, 0x41, 0x41, 0x41, 0x7F] // box for unknown
    }
}

/// Draw `text` with integer scale (pixel size of each font bit).
unsafe fn fb_draw_text(mut x: u32, y: u32, text: &[u8], scale: u32, color: u32) {
    let scale = scale.max(1);
    for &c in text {
        if c == b'\n' {
            continue;
        }
        let g = glyph5x7(c);
        for (col, bits) in g.iter().enumerate() {
            for row in 0..7u32 {
                if bits & (1 << row) != 0 {
                    unsafe {
                        fb_fill(
                            x + col as u32 * scale,
                            y + row * scale,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
        }
        // 5 px glyph + 1 px gap
        x = x.saturating_add(6 * scale);
        if x >= FB_W {
            break;
        }
    }
}

fn hex_u32(v: u32, out: &mut [u8; 10]) {
    out[0] = b'0';
    out[1] = b'x';
    const H: &[u8; 16] = b"0123456789ABCDEF";
    for i in 0..8 {
        out[2 + i] = H[((v >> (28 - i * 4)) & 0xF) as usize];
    }
}

fn fail_reason(msp: u32, reset: u32) -> &'static [u8] {
    let erased = msp == 0xFFFF_FFFF && reset == 0xFFFF_FFFF;
    let mostly_erased = msp == 0 || msp == 0xFFFF_FFFF || reset == 0 || reset == 0xFFFF_FFFF;
    if erased || mostly_erased {
        return b"NOR empty or erased (no image)";
    }
    let msp_ok = (0xC000_0000..=0xC400_0000).contains(&msp) && (msp & 7) == 0;
    if !msp_ok {
        return b"Invalid MSP (need SDRAM stack)";
    }
    if reset & 1 == 0 {
        return b"Reset vector missing Thumb bit";
    }
    let thr = reset & !1;
    if !(0x9000_0000..0x9800_0000).contains(&thr) {
        return b"Reset vector not in NOR XIP";
    }
    b"Vector table rejected"
}

/// Program LTDC layer 1 to scan the SDRAM framebuffer (same timing as OS).
unsafe fn ltdc_enable_fb() {
    unsafe {
        let apb3 = reg_read(RCC_APB3ENR);
        reg_write(RCC_APB3ENR, apb3 | RCC_APB3ENR_LTDCEN);
        let _ = reg_read(RCC_APB3ENR);

        let hsync = 48u32;
        let hbp = 40u32;
        let hfp = 40u32;
        let vsync = 1u32;
        let vbp = 31u32;
        let vfp = 13u32;
        let aw = FB_W;
        let ah = FB_H;
        let hsw = hsync - 1;
        let vsh = vsync - 1;
        let ahbp = hsync + hbp - 1;
        let avbp = vsync + vbp - 1;
        let aaw = hsync + hbp + aw - 1;
        let aah = vsync + vbp + ah - 1;
        let totalw = hsync + hbp + aw + hfp - 1;
        let totalh = vsync + vbp + ah + vfp - 1;

        ltdc_w(0x08, (hsw << 16) | vsh); // SSCR
        ltdc_w(0x0C, (ahbp << 16) | avbp); // BPCR
        ltdc_w(0x10, (aaw << 16) | aah); // AWCR
        ltdc_w(0x14, (totalw << 16) | totalh); // TWCR
        ltdc_w(0x2C, 0); // BCCR black

        let whst = hsync + hbp;
        let whsp = hsync + hbp + aw - 1;
        let wvst = vsync + vbp;
        let wvsp = vsync + vbp + ah - 1;
        ltdc_w(0x88, (whsp << 16) | whst); // L1WHPCR
        ltdc_w(0x8C, (wvsp << 16) | wvst); // L1WVPCR
        ltdc_w(0x94, 0); // L1PFCR ARGB8888
        ltdc_w(0x98, 0xFF); // L1CACR
        ltdc_w(0xA0, 0x607); // L1BFCR
        ltdc_w(0xAC, FB_BASE); // L1CFBAR
        ltdc_w(0xB0, (FB_STRIDE << 16) | (FB_STRIDE + 3)); // L1CFBLR
        ltdc_w(0xB4, FB_H); // L1CFBLNR
        ltdc_w(0x84, 1); // L1CR LEN
        ltdc_w(0x24, 1); // SRCR IMR
        ltdc_w(0x18, 0x2221); // GCR LTDCEN
        ltdc_w(0x24, 1); // reload again after enable (QEMU scanout)
    }
}

/// Full-screen fail UI when NOR has no bootable ArmOS image.
/// Black background, red text only — no chrome.
unsafe fn show_os_load_fail_screen(msp: u32, reset: u32) {
    unsafe {
        fb_fill_screen(COL_BLACK);

        fb_draw_text(40, 120, b"BOOT FAILED", 5, COL_RED);
        fb_draw_text(40, 200, b"No bootable OS on NOR flash", 2, COL_RED);
        fb_draw_text(40, 250, fail_reason(msp, reset), 2, COL_RED);

        let mut hex = [0u8; 10];
        let label_w = 6u32 * 6 * 2;
        hex_u32(msp, &mut hex);
        fb_draw_text(40, 310, b"MSP   ", 2, COL_RED);
        fb_draw_text(40 + label_w, 310, &hex, 2, COL_RED);

        hex_u32(reset, &mut hex);
        fb_draw_text(40, 350, b"RESET ", 2, COL_RED);
        fb_draw_text(40 + label_w, 350, &hex, 2, COL_RED);

        ltdc_enable_fb();
    }
}

fn os_vectors_look_valid(msp: u32, reset: u32) -> bool {
    // Stack in SDRAM window (bootloader maps 64 MiB @ 0xC0000000).
    let msp_ok = (0xC000_0000..=0xC400_0000).contains(&msp) && (msp & 7) == 0;
    // Reset in NOR XIP, Thumb bit set, not erased flash pattern.
    let reset_ok = (0x9000_0000..0x9800_0000).contains(&(reset & !1))
        && (reset & 1) == 1
        && reset != 0xFFFF_FFFF;
    msp_ok && reset_ok
}

/// Fatal: bad/empty NOR image. Stay in bootloader with ERR LED + panel.
unsafe fn halt_os_load_failed(msp: u32, reset: u32) -> ! {
    unsafe {
        led_set(LED_RUN, false);
        led_set(LED_DBG, false);
        led_set(LED_ERR, true);
        led_set(LED_PNC, false);
        show_os_load_fail_screen(msp, reset);
        loop {
            // Slow blink ERR so a stuck boot is visible on the LED window too.
            led_set(LED_ERR, true);
            for _ in 0..400_000 {
                core::hint::spin_loop();
            }
            led_set(LED_ERR, false);
            for _ in 0..400_000 {
                core::hint::spin_loop();
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Reset() -> ! {
    unsafe {
        /* .data / .bss in DTCM */
        let mut dst = core::ptr::addr_of_mut!(_sdata) as usize;
        let mut src = core::ptr::addr_of!(_sidata) as usize;
        let end = core::ptr::addr_of!(_edata) as usize;
        while dst < end {
            core::ptr::write_volatile(dst as *mut u32, core::ptr::read_volatile(src as *const u32));
            dst = dst.wrapping_add(4);
            src = src.wrapping_add(4);
        }
        let mut b = core::ptr::addr_of_mut!(_sbss) as usize;
        let bend = core::ptr::addr_of!(_ebss) as usize;
        while b < bend {
            core::ptr::write_volatile(b as *mut u32, 0);
            b = b.wrapping_add(4);
        }

        // 1) GPIO clocks first (required before MODER/AFR take effect on silicon)
        let ahb4 = reg_read(RCC_AHB4ENR);
        reg_write(
            RCC_AHB4ENR,
            ahb4 | RCC_AHB4ENR_GPIOCEN
                | RCC_AHB4ENR_GPIODEN
                | RCC_AHB4ENR_GPIOEEN
                | RCC_AHB4ENR_GPIOFEN
                | RCC_AHB4ENR_GPIOGEN
                | RCC_AHB4ENR_GPIOHEN
                | RCC_AHB4ENR_GPIOIEN,
        );
        let _ = reg_read(RCC_AHB4ENR);

        // Status LEDs early (DBG = “bringing up”)
        status_leds_init();
        led_set(LED_DBG, true);

        // 2) Board pinmux (FMC + QSPI + NOR reset)
        board_pinmux_init();

        // 3) Peripheral clocks
        let ahb3 = reg_read(RCC_AHB3ENR);
        reg_write(
            RCC_AHB3ENR,
            ahb3 | RCC_AHB3ENR_FMCEN | RCC_AHB3ENR_QSPIEN,
        );
        let _ = reg_read(RCC_AHB3ENR);

        // 4) External SDRAM
        fmc_sdram_init();

        // 5) NOR XIP window
        quadspi_enable_memmap();

        // 6) Handoff or fail UI
        jump_to_os();
    }
}

unsafe fn jump_to_os() -> ! {
    unsafe {
        let vtor = OS_VECTOR_TABLE as *const u32;
        let msp = core::ptr::read_volatile(vtor);
        let reset = core::ptr::read_volatile(vtor.add(1));

        if !os_vectors_look_valid(msp, reset) {
            halt_os_load_failed(msp, reset);
        }

        led_set(LED_DBG, false);
        led_set(LED_ERR, false);
        led_set(LED_RUN, true);

        const SCB_VTOR: *mut u32 = 0xE000_ED08 as *mut u32;
        core::ptr::write_volatile(SCB_VTOR, OS_VECTOR_TABLE);

        core::arch::asm!(
            "msr msp, {msp}",
            "bx {reset}",
            msp = in(reg) msp,
            reset = in(reg) reset,
            options(noreturn),
        );
    }
}
