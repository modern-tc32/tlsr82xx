#[cfg(feature = "chip-8258")]
use crate::mmio::{reg32, reg8};
use crate::pac;
#[cfg(feature = "chip-8258")]
use crate::regs8258::{
    FLD_IRQ_SYSTEM_TIMER, FLD_TMR0_EN, FLD_TMR0_MODE, REG_IRQ_MASK, REG_TMR0_CAPT, REG_TMR0_TICK,
    REG_TMR_CTRL, REG_TMR_STA,
};

#[cfg(feature = "chip-8258")]
const REG_SYSTEM_TICK_IRQ: usize = 0x0080_0744;
#[cfg(feature = "chip-8258")]
const REG_SYSTEM_TICK: usize = 0x0080_0740;
#[cfg(feature = "chip-8258")]
const REG_SYSTEM_TICK_MODE: usize = 0x0080_074c;
#[cfg(feature = "chip-8258")]
const REG_SYSTEM_TICK_CTRL: usize = 0x0080_074f;
#[cfg(feature = "chip-8258")]
const FLD_SYSTEM_IRQ_MASK: u8 = 1 << 1;
#[cfg(feature = "chip-8258")]
const FLD_SYSTEM_TICK_START: u8 = 1 << 0;
#[cfg(feature = "chip-8258")]
const FLD_SYSTEM_TICK_STOP: u8 = 1 << 1;

#[cfg(feature = "chip-8258")]
static mut TIMER0_PERIODIC_IRQ_ENABLED: bool = false;
#[cfg(feature = "chip-8258")]
static mut TIMER0_PERIODIC_IRQ_TICKS: u32 = 0;
#[cfg(feature = "chip-8258")]
static mut TIMER0_IRQ_COUNT: u32 = 0;
#[cfg(feature = "chip-8258")]
static mut TIMER0_IRQ_CALLBACK: Option<unsafe extern "C" fn()> = None;
#[cfg(feature = "chip-8258")]
static mut SYSTEM_TIMER_PERIODIC_IRQ_ENABLED: bool = false;
#[cfg(feature = "chip-8258")]
static mut SYSTEM_TIMER_PERIODIC_IRQ_TICKS: u32 = 0;
#[cfg(feature = "chip-8258")]
static mut SYSTEM_TIMER_IRQ_COUNT: u32 = 0;
#[cfg(feature = "chip-8258")]
static mut SYSTEM_TIMER_IRQ_CALLBACK: Option<unsafe extern "C" fn()> = None;

#[cfg(feature = "chip-8258")]
#[unsafe(no_mangle)]
pub static mut TLSR82XX_TIMER0_IRQ_PERIOD_TICKS: u32 = 0;
#[cfg(feature = "chip-8258")]
#[unsafe(no_mangle)]
pub static mut TLSR82XX_SYSTEM_TIMER_IRQ_PERIOD_TICKS: u32 = 0;

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

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn set_system_timer_irq_capture(tick: u32) {
    unsafe {
        core::ptr::write_volatile(reg32(REG_SYSTEM_TICK_IRQ), tick & !0x07);
    }
}

#[cfg(feature = "chip-8278")]
#[inline(always)]
pub fn set_system_timer_irq_capture(tick: u32) {
    unsafe {
        core::ptr::write_volatile((*pac::SystemTimer::ptr()).system_tick_irq().as_ptr(), tick);
    }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn system_timer_irq_capture() -> u32 {
    unsafe { core::ptr::read_volatile(reg32(REG_SYSTEM_TICK_IRQ).cast_const()) }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn system_timer_mode() -> u8 {
    unsafe { core::ptr::read_volatile(reg8(REG_SYSTEM_TICK_MODE).cast_const()) }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn system_timer_ctrl() -> u8 {
    unsafe { core::ptr::read_volatile(reg8(REG_SYSTEM_TICK_CTRL).cast_const()) }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn system_timer_value() -> u32 {
    unsafe { core::ptr::read_volatile(reg32(REG_SYSTEM_TICK).cast_const()) }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn clear_system_timer_irq_status() {
    crate::interrupt::acknowledge_irq(crate::interrupt::Irq::SystemTimer);
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn start_system_timer() {
    unsafe {
        core::ptr::write_volatile(reg8(REG_SYSTEM_TICK_CTRL), FLD_SYSTEM_TICK_START);
    }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn stop_system_timer() {
    unsafe {
        core::ptr::write_volatile(reg8(REG_SYSTEM_TICK_CTRL), FLD_SYSTEM_TICK_STOP);
    }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn enable_system_timer_irq() {
    unsafe {
        let irq_mask = reg32(REG_IRQ_MASK);
        core::ptr::write_volatile(
            irq_mask,
            core::ptr::read_volatile(irq_mask.cast_const()) | FLD_IRQ_SYSTEM_TIMER,
        );

        let tick_mode = reg8(REG_SYSTEM_TICK_MODE);
        core::ptr::write_volatile(
            tick_mode,
            core::ptr::read_volatile(tick_mode.cast_const()) | FLD_SYSTEM_IRQ_MASK,
        );
    }
}

#[cfg(feature = "chip-8278")]
#[inline(always)]
pub fn enable_system_timer_irq() {
    unsafe {
        (*pac::SystemTimer::ptr())
            .system_tick_mode()
            .modify(|r, w| w.bits(r.bits() | (1 << 1)));
    }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn disable_system_timer_irq() {
    unsafe {
        let irq_mask = reg32(REG_IRQ_MASK);
        core::ptr::write_volatile(
            irq_mask,
            core::ptr::read_volatile(irq_mask.cast_const()) & !FLD_IRQ_SYSTEM_TIMER,
        );

        let tick_mode = reg8(REG_SYSTEM_TICK_MODE);
        core::ptr::write_volatile(
            tick_mode,
            core::ptr::read_volatile(tick_mode.cast_const()) & !FLD_SYSTEM_IRQ_MASK,
        );
    }
}

#[cfg(feature = "chip-8258")]
pub fn configure_system_timer_periodic_irq(period_ticks: u32) {
    let first_compare = clock_time().wrapping_add(period_ticks) & !0x07;
    unsafe {
        core::ptr::write_volatile(&raw mut SYSTEM_TIMER_PERIODIC_IRQ_ENABLED, true);
        core::ptr::write_volatile(&raw mut SYSTEM_TIMER_PERIODIC_IRQ_TICKS, period_ticks);
        core::ptr::write_volatile(&raw mut SYSTEM_TIMER_IRQ_COUNT, 0);
        core::ptr::write_volatile(&raw mut TLSR82XX_SYSTEM_TIMER_IRQ_PERIOD_TICKS, period_ticks);
    }
    clear_system_timer_irq_status();
    set_system_timer_irq_capture(first_compare);
    enable_system_timer_irq();
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn system_timer_periodic_irq_enabled() -> bool {
    unsafe { core::ptr::read_volatile(&raw const SYSTEM_TIMER_PERIODIC_IRQ_ENABLED) }
}

#[cfg(feature = "chip-8258")]
#[unsafe(link_section = ".ram_code")]
pub fn system_timer_periodic_irq_fired() {
    unsafe {
        let count = core::ptr::read_volatile(&raw const SYSTEM_TIMER_IRQ_COUNT);
        core::ptr::write_volatile(&raw mut SYSTEM_TIMER_IRQ_COUNT, count.wrapping_add(1));
    }
}

#[cfg(feature = "chip-8258")]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code")]
pub extern "C" fn tlsr82xx_system_timer_irq_tick() {
    system_timer_periodic_irq_fired();
    unsafe {
        if let Some(callback) = SYSTEM_TIMER_IRQ_CALLBACK {
            callback();
        }
    }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn system_timer_irq_count() -> u32 {
    unsafe { core::ptr::read_volatile(&raw const SYSTEM_TIMER_IRQ_COUNT) }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn system_timer_irq_phase() -> bool {
    (system_timer_irq_count() & 1) != 0
}

#[cfg(feature = "chip-8258")]
pub fn register_system_timer_irq_callback(callback: unsafe extern "C" fn()) {
    unsafe {
        core::ptr::write_volatile(&raw mut SYSTEM_TIMER_IRQ_CALLBACK, Some(callback));
    }
}

#[cfg(feature = "chip-8258")]
pub fn unregister_system_timer_irq_callback() {
    unsafe {
        core::ptr::write_volatile(&raw mut SYSTEM_TIMER_IRQ_CALLBACK, None);
    }
}

#[cfg(feature = "chip-8278")]
#[inline(always)]
pub fn disable_system_timer_irq() {
    unsafe {
        (*pac::SystemTimer::ptr())
            .system_tick_mode()
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
    unsafe {
        if let Some(callback) = TIMER0_IRQ_CALLBACK {
            callback();
        }
    }
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

#[cfg(feature = "chip-8258")]
pub fn register_timer0_irq_callback(callback: unsafe extern "C" fn()) {
    unsafe {
        core::ptr::write_volatile(&raw mut TIMER0_IRQ_CALLBACK, Some(callback));
    }
}

#[cfg(feature = "chip-8258")]
pub fn unregister_timer0_irq_callback() {
    unsafe {
        core::ptr::write_volatile(&raw mut TIMER0_IRQ_CALLBACK, None);
    }
}
