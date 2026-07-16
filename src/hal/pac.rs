//! STM32H745 (Cortex-M7) peripheral access via the `stm32h7` PAC.

pub use stm32h7::stm32h745cm7 as device;

/// Returns a pointer to the USART1 peripheral registers.
#[inline(always)]
pub fn usart1() -> &'static device::usart1::RegisterBlock {
    unsafe { &*device::USART1::ptr() }
}

/// Returns a pointer to the RCC peripheral registers.
#[inline(always)]
pub fn rcc() -> &'static device::rcc::RegisterBlock {
    unsafe { &*device::RCC::ptr() }
}

/// Returns a pointer to the LTDC peripheral registers.
#[inline(always)]
pub fn ltdc() -> &'static device::ltdc::RegisterBlock {
    unsafe { &*device::LTDC::ptr() }
}