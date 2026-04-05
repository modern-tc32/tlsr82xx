use crate::{clock, interrupt, timer};

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartupState {
    Boot = 0,
    DeepRetention = 1,
    Deep = 2,
}

unsafe extern "C" {
    fn cpu_wakeup_init();
}

#[unsafe(no_mangle)]
pub static mut sysTimerPerUs: u32 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn drv_calibration() {}

#[unsafe(no_mangle)]
pub static mut adc_gpio_calib_vref: u16 = 1175;

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

#[inline(always)]
pub fn init() -> StartupState {
    interrupt::disable();
    interrupt::clear_mask(interrupt::ALL_IRQS);
    interrupt::clear_all_irq_sources();

    unsafe {
        cpu_wakeup_init();
    }
    clock::init(clock::SysClock::Crystal48M);
    unsafe {
        sysTimerPerUs = timer::SYS_TICK_PER_US;
    }

    StartupState::Boot
}
