use crate::pac;
#[cfg(feature = "chip-8258")]
use crate::mmio::{reg32, reg8};
#[cfg(feature = "chip-8258")]
use crate::regs8258::{
    FLD_TMR0_EN, FLD_TMR0_MODE, REG_TMR0_CAPT, REG_TMR0_TICK, REG_TMR_CTRL, REG_TMR_STA,
};

#[inline(always)]
pub fn clock_time() -> u32 {
    unsafe { (*pac::SystemTimer::ptr()).system_tick().read().bits() }
}

#[inline(always)]
pub fn clock_time_exceed_ticks(reference: u32, ticks: u32) -> bool {
    clock_time().wrapping_sub(reference) > ticks
}

#[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
pub const SYS_TICK_PER_US: u32 = 16;

#[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
#[inline(always)]
pub fn clock_time_exceed_us(reference: u32, microseconds: u32) -> bool {
    clock_time_exceed_ticks(reference, microseconds.wrapping_mul(SYS_TICK_PER_US))
}

#[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
#[inline(always)]
pub fn set_system_timer_irq_capture(tick: u32) {
    unsafe {
        core::ptr::write_volatile(
            (*pac::SystemTimer::ptr()).system_tick_irq().as_ptr(),
            tick,
        );
    }
}

#[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
#[inline(always)]
pub fn enable_system_timer_irq() {
    unsafe {
        (*pac::SystemTimer::ptr())
            .system_tick_mode()
            .modify(|r, w| w.bits(r.bits() | (1 << 1)));
        (*pac::SystemTimer::ptr())
            .system_tick_ctrl()
            .modify(|r, w| w.bits(r.bits() | (1 << 1)));
    }
}

#[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
#[inline(always)]
pub fn disable_system_timer_irq() {
    unsafe {
        (*pac::SystemTimer::ptr())
            .system_tick_mode()
            .modify(|r, w| w.bits(r.bits() & !(1 << 1)));
        (*pac::SystemTimer::ptr())
            .system_tick_ctrl()
            .modify(|r, w| w.bits(r.bits() & !(1 << 1)));
    }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn set_timer0_tick(tick: u32) {
    unsafe {
        core::ptr::write_volatile(reg32(REG_TMR0_TICK), tick);
    }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn set_timer0_irq_capture(tick: u32) {
    unsafe {
        core::ptr::write_volatile(reg32(REG_TMR0_CAPT), tick);
    }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn set_timer0_mode_sysclk() {
    unsafe {
        core::ptr::write_volatile(reg8(REG_TMR_STA), 1);
        let reg = reg32(REG_TMR_CTRL);
        let value = (core::ptr::read_volatile(reg.cast_const()) & !FLD_TMR0_MODE) | FLD_TMR0_EN;
        core::ptr::write_volatile(reg, value);
    }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn enable_timer0() {
    unsafe {
        let reg = reg32(REG_TMR_CTRL);
        let value = core::ptr::read_volatile(reg.cast_const()) | FLD_TMR0_EN;
        core::ptr::write_volatile(reg, value);
    }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn disable_timer0() {
    unsafe {
        let reg = reg32(REG_TMR_CTRL);
        let value = core::ptr::read_volatile(reg.cast_const()) & !FLD_TMR0_EN;
        core::ptr::write_volatile(reg, value);
    }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn clear_timer0_status() {
    unsafe {
        core::ptr::write_volatile(reg8(REG_TMR_STA), 1);
    }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn is_timer0_pending() -> bool {
    unsafe { (core::ptr::read_volatile(reg8(REG_TMR_STA).cast_const()) & 1) != 0 }
}
