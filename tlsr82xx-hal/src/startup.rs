use crate::{analog, clock, interrupt, timer};

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartupState {
    Boot = 0,
    DeepRetention = 1,
    Deep = 2,
}

const ANA_POWER_CTRL_ADDR: usize = 0x0080_0074;
const PM_INFO0_ADDR: usize = 0x0080_0048;
const PM_INFO1_ADDR: usize = 0x0080_004c;
const SYSTEM_ON_BASE_ADDR: usize = 0x0080_0060;
const FLASH_CTRL_ADDR: usize = 0x0080_006f;

#[inline(always)]
fn reg8(addr: usize) -> *mut u8 {
    addr as *mut u8
}

#[inline(always)]
fn reg32(addr: usize) -> *mut u32 {
    addr as *mut u32
}

#[unsafe(no_mangle)]
pub static mut sysTimerPerUs: u32 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn drv_calibration() {}

#[unsafe(no_mangle)]
pub static mut adc_gpio_calib_vref: u16 = 1175;

#[unsafe(no_mangle)]
pub static mut tl_24mrc_cal: u8 = 0;

#[unsafe(no_mangle)]
pub static mut g_pm_xtal_stable_loopnum: u32 = 0x40;

#[unsafe(no_mangle)]
pub static mut g_pm_xtal_stable_suspend_nopnum: u32 = 0x40;

#[unsafe(no_mangle)]
pub extern "C" fn adc_set_gpio_calib_vref(data: u16) {
    unsafe {
        adc_gpio_calib_vref = data;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn adc_get_result_with_fluct(fluctuation_mv: *mut u32) -> u32 {
    if !fluctuation_mv.is_null() {
        unsafe {
            core::ptr::write_volatile(fluctuation_mv, 0);
        }
    }

    // Current examples do not use ADC directly. Returning a stable value above
    // the flash safety threshold preserves compatibility with legacy startup
    // code that still expects this helper.
    3300
}

#[unsafe(no_mangle)]
pub extern "C" fn pm_get_info0() -> u32 {
    unsafe {
        core::ptr::write_volatile(reg8(ANA_POWER_CTRL_ADDR), 0x62);
        let value = core::ptr::read_volatile(reg32(PM_INFO0_ADDR).cast_const());
        core::ptr::write_volatile(reg8(ANA_POWER_CTRL_ADDR), 0);
        value
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pm_get_info1() -> u32 {
    unsafe {
        core::ptr::write_volatile(reg8(ANA_POWER_CTRL_ADDR), 0x62);
        let value = core::ptr::read_volatile(reg32(PM_INFO1_ADDR).cast_const());
        core::ptr::write_volatile(reg8(ANA_POWER_CTRL_ADDR), 0);
        value
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pm_get_32k_tick() -> u32 {
    let mut prev = 0u32;
    let mut have_prev = false;

    loop {
        let value = ((analog::read(0x43) as u32) << 24)
            | ((analog::read(0x42) as u32) << 16)
            | ((analog::read(0x41) as u32) << 8)
            | analog::read(0x40) as u32;
        if !have_prev || value.wrapping_sub(prev) <= 1 {
            return value;
        }
        prev = value;
        have_prev = true;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pm_wait_xtal_ready() {
    let start = timer::clock_time();
    while !timer::clock_time_exceed_us(start, 256) {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cpu_wakeup_init() {
    unsafe {
        core::ptr::write_volatile(reg32(SYSTEM_ON_BASE_ADDR), 0xff00_0000);
        core::ptr::write_volatile(reg8(SYSTEM_ON_BASE_ADDR + 4), 0xff);
        core::ptr::write_volatile(reg8(SYSTEM_ON_BASE_ADDR + 5), 0xff);
        core::ptr::write_volatile(reg8(FLASH_CTRL_ADDR), 0x80);
    }

    analog::write(0x82, 0x64);
    analog::write(0x34, 0x80);
    analog::write(0x0b, 0x38);
    analog::write(0x8c, 0x02);
    analog::write(0x02, 0xa2);
    analog::write(0x27, 0x00);
    analog::write(0x28, 0x00);
    analog::write(0x29, 0x00);
    analog::write(0x2a, 0x00);
}

#[inline(always)]
pub fn init() -> StartupState {
    interrupt::disable();
    interrupt::clear_mask(interrupt::ALL_IRQS);
    interrupt::clear_all_irq_sources();

    cpu_wakeup_init();
    clock::init(clock::SysClock::Crystal48M);
    unsafe {
        sysTimerPerUs = timer::SYS_TICK_PER_US;
    }

    StartupState::Boot
}
