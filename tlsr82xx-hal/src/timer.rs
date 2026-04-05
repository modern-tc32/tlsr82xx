use crate::pac;

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
