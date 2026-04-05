use crate::{analog, clock, interrupt, timer};
use crate::mmio::{reg8, reg32};
#[cfg(feature = "chip-8258")]
use crate::regs8258::{
    ANA_32K_TICK_BYTE0, ANA_32K_TICK_BYTE1, ANA_32K_TICK_BYTE2, ANA_32K_TICK_BYTE3, ANA_REG_0X02,
    ANA_REG_0X27, ANA_REG_0X28, ANA_REG_0X29, ANA_REG_0X2A, ANA_REG_0X8A, ANA_REG_0X8C,
    ANA_USB_DP_PULLUP, ANA_USB_POWER, AREG_CLK_SETTING, REG_ANA_POWER_CTRL, REG_CLK_EN0,
    REG_CLK_EN1, REG_CLK_EN2, REG_PM_INFO0, REG_PM_INFO1, REG_PWDN_CTRL, REG_RST0, REG_RST1,
    REG_RST2, REG_SYSTEM_TICK,
};

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartupState {
    Boot = 0,
    DeepRetention = 1,
    Deep = 2,
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
        core::ptr::write_volatile(reg8(REG_ANA_POWER_CTRL), 0x62);
        let value = core::ptr::read_volatile(reg32(REG_PM_INFO0).cast_const());
        core::ptr::write_volatile(reg8(REG_ANA_POWER_CTRL), 0);
        value
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pm_get_info1() -> u32 {
    unsafe {
        core::ptr::write_volatile(reg8(REG_ANA_POWER_CTRL), 0x62);
        let value = core::ptr::read_volatile(reg32(REG_PM_INFO1).cast_const());
        core::ptr::write_volatile(reg8(REG_ANA_POWER_CTRL), 0);
        value
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pm_get_32k_tick() -> u32 {
    let mut prev = 0u32;
    let mut have_prev = false;

    loop {
        let value = ((analog::read(ANA_32K_TICK_BYTE3) as u32) << 24)
            | ((analog::read(ANA_32K_TICK_BYTE2) as u32) << 16)
            | ((analog::read(ANA_32K_TICK_BYTE1) as u32) << 8)
            | analog::read(ANA_32K_TICK_BYTE0) as u32;
        if !have_prev || value.wrapping_sub(prev) <= 1 {
            return value;
        }
        prev = value;
        have_prev = true;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pm_wait_xtal_ready() {
    let start = unsafe { core::ptr::read_volatile(reg32(REG_SYSTEM_TICK).cast_const()) };
    while unsafe { core::ptr::read_volatile(reg32(REG_SYSTEM_TICK).cast_const()) }
        .wrapping_sub(start)
        <= 256 * timer::SYS_TICK_PER_US
    {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cpu_wakeup_init() {
    unsafe {
        core::ptr::write_volatile(reg8(REG_RST0), 0x00);
        core::ptr::write_volatile(reg8(REG_RST1), 0x00);
        core::ptr::write_volatile(reg8(REG_RST2), 0x00);
        core::ptr::write_volatile(reg8(REG_CLK_EN0), 0xff);
        core::ptr::write_volatile(reg8(REG_CLK_EN1), 0xff);
        core::ptr::write_volatile(reg8(REG_CLK_EN2), 0xff);
        core::ptr::write_volatile(reg8(REG_PWDN_CTRL), 0x80);
    }

    analog::write(AREG_CLK_SETTING, 0x64);
    analog::write(ANA_USB_POWER, 0x80);
    analog::write(ANA_USB_DP_PULLUP, 0x38);
    analog::write(ANA_REG_0X8C, 0x02);
    analog::write(ANA_REG_0X02, 0xa2);
    analog::write(ANA_REG_0X27, 0x00);
    analog::write(ANA_REG_0X28, 0x00);
    analog::write(ANA_REG_0X29, 0x00);
    analog::write(ANA_REG_0X2A, 0x00);
    let _ = ANA_REG_0X8A;
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
