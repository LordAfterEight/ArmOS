//! GPIO helpers built on the `stm32h7` PAC.

use crate::board::GpioPin;

use super::pac::device;

pub fn configure_af_pin(gpio: &device::gpioa::RegisterBlock, pin: &GpioPin) {
    set_moder_alternate(gpio, pin.pin);
    set_afr(gpio, pin.pin, pin.alternate);
}

fn set_moder_alternate(gpio: &device::gpioa::RegisterBlock, pin: u8) {
    match pin {
        0 => gpio.moder().modify(|_, w| w.moder0().alternate()),
        1 => gpio.moder().modify(|_, w| w.moder1().alternate()),
        2 => gpio.moder().modify(|_, w| w.moder2().alternate()),
        3 => gpio.moder().modify(|_, w| w.moder3().alternate()),
        4 => gpio.moder().modify(|_, w| w.moder4().alternate()),
        5 => gpio.moder().modify(|_, w| w.moder5().alternate()),
        6 => gpio.moder().modify(|_, w| w.moder6().alternate()),
        7 => gpio.moder().modify(|_, w| w.moder7().alternate()),
        8 => gpio.moder().modify(|_, w| w.moder8().alternate()),
        9 => gpio.moder().modify(|_, w| w.moder9().alternate()),
        10 => gpio.moder().modify(|_, w| w.moder10().alternate()),
        11 => gpio.moder().modify(|_, w| w.moder11().alternate()),
        12 => gpio.moder().modify(|_, w| w.moder12().alternate()),
        13 => gpio.moder().modify(|_, w| w.moder13().alternate()),
        14 => gpio.moder().modify(|_, w| w.moder14().alternate()),
        15 => gpio.moder().modify(|_, w| w.moder15().alternate()),
        _ => return,
    };
}

fn set_afr(gpio: &device::gpioa::RegisterBlock, pin: u8, alternate: u8) {
    match pin {
        0 => gpio.afrl().modify(|_, w| unsafe { w.afr0().bits(alternate) }),
        1 => gpio.afrl().modify(|_, w| unsafe { w.afr1().bits(alternate) }),
        2 => gpio.afrl().modify(|_, w| unsafe { w.afr2().bits(alternate) }),
        3 => gpio.afrl().modify(|_, w| unsafe { w.afr3().bits(alternate) }),
        4 => gpio.afrl().modify(|_, w| unsafe { w.afr4().bits(alternate) }),
        5 => gpio.afrl().modify(|_, w| unsafe { w.afr5().bits(alternate) }),
        6 => gpio.afrl().modify(|_, w| unsafe { w.afr6().bits(alternate) }),
        7 => gpio.afrl().modify(|_, w| unsafe { w.afr7().bits(alternate) }),
        8 => gpio.afrh().modify(|_, w| unsafe { w.afr8().bits(alternate) }),
        9 => gpio.afrh().modify(|_, w| unsafe { w.afr9().bits(alternate) }),
        10 => gpio.afrh().modify(|_, w| unsafe { w.afr10().bits(alternate) }),
        11 => gpio.afrh().modify(|_, w| unsafe { w.afr11().bits(alternate) }),
        12 => gpio.afrh().modify(|_, w| unsafe { w.afr12().bits(alternate) }),
        13 => gpio.afrh().modify(|_, w| unsafe { w.afr13().bits(alternate) }),
        14 => gpio.afrh().modify(|_, w| unsafe { w.afr14().bits(alternate) }),
        15 => gpio.afrh().modify(|_, w| unsafe { w.afr15().bits(alternate) }),
        _ => return,
    };
}