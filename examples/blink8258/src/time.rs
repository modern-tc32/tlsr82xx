use core::ptr;

const REG_SYSTEM_TICK: *const u32 = 0x800740 as *const u32;
const SYS_TICK_PER_US: u32 = 16;

pub fn clock_time() -> u32 {
    unsafe { ptr::read_volatile(REG_SYSTEM_TICK) }
}

pub fn clock_time_exceed(reference: u32, microseconds: u32) -> bool {
    clock_time().wrapping_sub(reference) > microseconds.wrapping_mul(SYS_TICK_PER_US)
}
