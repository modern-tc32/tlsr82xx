use crate::pac;
#[cfg(feature = "chip-8258")]
use crate::mmio::{reg32, reg8};
#[cfg(feature = "chip-8258")]
use crate::regs8258::{
    FLD_TMR0_EN, FLD_TMR0_MODE, REG_TMR0_CAPT, REG_TMR0_TICK, REG_TMR_CTRL, REG_TMR_STA,
};

#[cfg(feature = "chip-8258")]
static mut TIMER0_PERIODIC_IRQ_ENABLED: bool = false;
#[cfg(feature = "chip-8258")]
static mut TIMER0_PERIODIC_IRQ_TICKS: u32 = 0;
#[cfg(feature = "chip-8258")]
static mut TIMER0_IRQ_COUNT: u32 = 0;

#[cfg(feature = "chip-8258")]
#[unsafe(no_mangle)]
pub static mut TLSR82XX_TIMER0_IRQ_PERIOD_TICKS: u32 = 0;

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

#[cfg(feature = "chip-8258")]
pub fn configure_timer0_periodic_irq(period_ticks: u32) {
    unsafe {
        core::ptr::write_volatile(&raw mut TIMER0_PERIODIC_IRQ_ENABLED, true);
        core::ptr::write_volatile(&raw mut TIMER0_PERIODIC_IRQ_TICKS, period_ticks);
        core::ptr::write_volatile(&raw mut TIMER0_IRQ_COUNT, 0);
        core::ptr::write_volatile(&raw mut TLSR82XX_TIMER0_IRQ_PERIOD_TICKS, period_ticks);
    }

    clear_timer0_status();
    set_timer0_mode_sysclk();
    rearm_timer0_periodic_irq();
}

#[cfg(feature = "chip-8258")]
#[unsafe(link_section = ".ram_code")]
pub fn rearm_timer0_periodic_irq() {
    let period_ticks = unsafe { core::ptr::read_volatile(&raw const TIMER0_PERIODIC_IRQ_TICKS) };
    set_timer0_tick(0);
    set_timer0_irq_capture(period_ticks);
    enable_timer0();
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn timer0_periodic_irq_enabled() -> bool {
    unsafe { core::ptr::read_volatile(&raw const TIMER0_PERIODIC_IRQ_ENABLED) }
}

#[cfg(feature = "chip-8258")]
#[unsafe(link_section = ".ram_code")]
pub fn timer0_periodic_irq_fired() {
    unsafe {
        let count = core::ptr::read_volatile(&raw const TIMER0_IRQ_COUNT);
        core::ptr::write_volatile(&raw mut TIMER0_IRQ_COUNT, count.wrapping_add(1));
    }
}

#[cfg(feature = "chip-8258")]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code")]
pub extern "C" fn tlsr82xx_timer0_irq_tick() {
    timer0_periodic_irq_fired();
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn timer0_irq_count() -> u32 {
    unsafe { core::ptr::read_volatile(&raw const TIMER0_IRQ_COUNT) }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn timer0_irq_phase() -> bool {
    (timer0_irq_count() & 1) != 0
}
