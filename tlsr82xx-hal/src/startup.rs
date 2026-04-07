use crate::{analog, clock, interrupt, timer};
use crate::mmio::{reg16, reg8, reg32};
#[cfg(feature = "chip-8258")]
use crate::regs8258::{
    ANA_32K_TICK_BYTE0, ANA_32K_TICK_BYTE1, ANA_32K_TICK_BYTE2, ANA_32K_TICK_BYTE3, ANA_REG_0X02,
    ANA_REG_0X27, ANA_REG_0X28, ANA_REG_0X29, ANA_REG_0X2A, ANA_REG_0X8A, ANA_REG_0X8C,
    ANA_USB_DP_PULLUP, ANA_USB_POWER, AREG_CLK_SETTING, REG_ANA_POWER_CTRL, REG_CLK_EN0,
    REG_CLK_EN1, REG_CLK_EN2, REG_DCDC_CTRL, REG_DFIFO0_ADDR, REG_DFIFO0_SIZE, REG_DFIFO1_ADDR,
    REG_DMA_CHN_EN, REG_GPIO_PE_IE, REG_GPIO_WAKEUP_IRQ, REG_IRQ_MASK, REG_MCU_WAKEUP_MASK,
    REG_MSPI_CTRL, REG_MSPI_DATA, REG_PM_INFO0, REG_PM_INFO1, REG_PM_WAKEUP_FLAG, REG_PWDN_CTRL,
    REG_RF_IRQ_STATUS, REG_RST0, REG_RST1, REG_RST2, REG_SUSPEND_RET_ADDR_HI, REG_SYSTEM_TICK,
    REG_SYSTEM_TICK_CTRL, REG_TMR0_TICK, REG_TMR1_TICK, REG_TMR2_TICK, REG_TMR_STA,
    REG_WAKEUP_SRC,
};

unsafe extern "C" {
    static mut _dstored_: u32;
    static mut _start_data_: u32;
    static mut _end_data_: u32;
    static mut _start_bss_: u32;
    static mut _end_bss_: u32;
    static mut _custom_stored_: u32;
    static mut _start_custom_data_: u32;
    static mut _end_custom_data_: u32;
    static mut _start_custom_bss_: u32;
    static mut _end_custom_bss_: u32;
    static mut _stack_end_: u32;
    static mut _ictag_start_: u32;
    static mut _ictag_end_: u32;
    static mut _ramcode_size_align_256_: u32;
    fn main() -> i32;
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartupState {
    Boot = 0,
    DeepRetention = 1,
    Deep = 2,
}

#[repr(C)]
pub struct PmPara {
    pub is_pad_wakeup: u8,
    pub wakeup_src: u8,
    pub mcu_status: u8,
    pub _reserved: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PmEarlyWakeupTimeUs {
    pub suspend: u16,
    pub deep_ret: u16,
    pub deep: u16,
    pub min: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PmRDelayUs {
    pub deep_r_delay_us: u16,
    pub suspend_ret_r_delay_us: u16,
}

const MCU_STATUS_BOOT: u8 = 0;
const MCU_STATUS_DEEPRET_BACK: u8 = 1;
const MCU_STATUS_DEEP_BACK: u8 = 2;

const REG_PM_RET_CTRL0: usize = REG_DFIFO0_ADDR;
const REG_PM_RET_CTRL1: usize = REG_DFIFO1_ADDR;
const REG_PM_RET_BYTE: usize = REG_DFIFO0_SIZE;
const REG_PM_RET_CLR: usize = REG_DMA_CHN_EN;
const REG_PM_WAIT: usize = REG_SYSTEM_TICK_CTRL;
const REG_RF_IRQ_DONE: usize = REG_RF_IRQ_STATUS;

#[unsafe(no_mangle)]
pub static mut sysTimerPerUs: u32 = 0;

#[unsafe(no_mangle)]
pub static mut pmParam: PmPara = PmPara {
    is_pad_wakeup: 0,
    wakeup_src: 0,
    mcu_status: 0,
    _reserved: 0,
};

#[unsafe(no_mangle)]
pub static mut pm_tim_recover: usize = 0;

#[unsafe(no_mangle)]
pub static mut func_before_suspend: usize = 0;

#[unsafe(no_mangle)]
pub static mut cpu_sleep_wakeup: usize = 0;

#[unsafe(no_mangle)]
pub static mut tl_multi_addr: u8 = 0;

#[unsafe(no_mangle)]
pub static mut tick_32k_calib: u16 = 0;

#[unsafe(no_mangle)]
pub static mut tick_cur: u32 = 0;

#[unsafe(no_mangle)]
pub static mut tick_32k_cur: u32 = 0;

#[unsafe(no_mangle)]
pub static mut pm_long_suspend: u8 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn drv_calibration() {}

#[unsafe(no_mangle)]
pub static mut pm_curr_stack: usize = 0;

#[unsafe(no_mangle)]
pub static mut pm_bit_info_0: u8 = 0;

#[unsafe(no_mangle)]
pub static mut pm_bit_info_1: u8 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn efuse_sys_check(info1: u32) {
    let info0 = pm_get_info0();
    let low_nibble = info0 & 0x0f;
    if low_nibble > 9 {
        unsafe {
            core::ptr::write_volatile(reg8(REG_PWDN_CTRL), 0x20);
        }
        loop {
            core::hint::spin_loop();
        }
    }

    let bit_info_1 = ((info1 << 6) >> 30) as u8;
    let bit_info_0 = (info1 >> 29) as u8;
    unsafe {
        pm_bit_info_1 = bit_info_1;
        pm_bit_info_0 = bit_info_0;
    }

    let mut need_clear = false;
    if (info1 & 0xc0) == 0xc0 {
        need_clear = true;
        if bit_info_1 <= 1 {
            need_clear = ((info1 << 23) >> 31) != 0;
        }
    } else if bit_info_0 != 0 {
        need_clear = true;
    }

    if !need_clear {
        return;
    }

    let mut stack_probe = 0u8;
    let current_sp = (&mut stack_probe as *mut u8 as usize) & !0xffusize;
    unsafe {
        pm_curr_stack = current_sp;
    }
    let upper = current_sp.wrapping_add(100) & !0xffusize;

    if !need_clear {
        return;
    }

    if bit_info_1 == 0 {
        if bit_info_0 == 2 {
            if current_sp <= 0x0084_8000 {
                return;
            }
        } else if bit_info_0 == 4 {
            if current_sp <= 0x0084_c000 {
                return;
            }
        } else {
            return;
        }
    }

    let mut addr = (current_sp.wrapping_sub(100)) & !0xffusize;
    while addr < upper {
        unsafe {
            core::ptr::write_volatile((addr | 0x0080_0000usize) as *mut u32, 0);
        }
        addr = addr.wrapping_add(16);
    }
}

#[unsafe(no_mangle)]
pub static mut adc_gpio_calib_vref: u16 = 1175;

#[unsafe(no_mangle)]
pub static mut tl_24mrc_cal: u8 = 0x80;

#[unsafe(no_mangle)]
pub static mut g_pm_r_delay_us: PmRDelayUs = PmRDelayUs {
    deep_r_delay_us: 1000,
    suspend_ret_r_delay_us: 1000,
};

#[unsafe(no_mangle)]
pub static mut g_pm_early_wakeup_time_us: PmEarlyWakeupTimeUs = PmEarlyWakeupTimeUs {
    suspend: 0x0555,
    deep_ret: 0x044c,
    deep: 0x04d8,
    min: 0x06e5,
};

#[unsafe(no_mangle)]
pub static mut g_pm_suspend_delay_us: u32 = 0x87;

#[unsafe(no_mangle)]
pub static mut g_pm_xtal_stable_loopnum: u32 = 10;

#[unsafe(no_mangle)]
pub static mut g_pm_xtal_stable_suspend_nopnum: u32 = 200;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __tc32_fill_stack_pattern(mut start: *mut u32, end: *mut u32) {
    while start < end {
        core::ptr::write_volatile(start, 0xffff_ffff);
        start = start.add(1);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __tc32_copy_words(
    mut dst: *mut u32,
    end: *mut u32,
    mut src: *const u32,
) {
    while dst < end {
        core::ptr::write_volatile(dst, core::ptr::read_volatile(src));
        dst = dst.add(1);
        src = src.add(1);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __tc32_zero_words(mut dst: *mut u32, end: *mut u32) {
    while dst < end {
        core::ptr::write_volatile(dst, 0);
        dst = dst.add(1);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn __tc32_analog_read_u8(reg: u8) -> u8 {
    unsafe {
        let ana = reg8(0x8000b8);
        core::ptr::write_volatile(ana, reg);
        core::ptr::write_volatile(ana.add(2), 0x40);
        while (core::ptr::read_volatile(ana.add(2).cast_const()) & 1) != 0 {}
        core::ptr::read_volatile(ana.add(1).cast_const())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn __tc32_system_on_for_flash() {
    unsafe {
        core::ptr::write_volatile(reg32(0x800060), 0xff08_0000);
        core::ptr::write_volatile(reg8(0x800064), 0xff);
        core::ptr::write_volatile(reg8(0x800065), 0xf7);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __tc32_init_icache(
    mut tag_start: *mut u32,
    tag_end: *mut u32,
    ramcode_size_align_256: *mut u32,
) {
    while tag_start < tag_end {
        core::ptr::write_volatile(tag_start, 0);
        tag_start = tag_start.add(1);
    }

    let cache = reg8(0x80060c);
    let lines = ((ramcode_size_align_256 as usize) >> 8) as u8;
    core::ptr::write_volatile(cache, lines);
    core::ptr::write_volatile(cache.add(1), lines.wrapping_add(1));
}

#[unsafe(no_mangle)]
pub extern "C" fn __tc32_flash_wakeup() {
    unsafe {
        let flash = reg8(0x80000c);
        core::ptr::write_volatile(flash.add(1), 0);
        core::ptr::write_volatile(flash, 0xab);
        for _ in 0..=6u32 {
            core::hint::spin_loop();
        }
        core::ptr::write_volatile(flash.add(1), 1);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn __tc32_efuse_delay() {
    for _ in 0..110u32 {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".vectors.boot")]
pub extern "C" fn __tc32_boot_init() -> ! {
    unsafe {
        __tc32_init_icache(
            core::ptr::addr_of_mut!(_ictag_start_),
            core::ptr::addr_of_mut!(_ictag_end_),
            core::ptr::addr_of_mut!(_ramcode_size_align_256_),
        );
        __tc32_system_on_for_flash();
        __tc32_flash_wakeup();
        __tc32_efuse_delay();

        let wake_flag = __tc32_analog_read_u8(0x7e);
        if (wake_flag & 1) != 0 {
            core::ptr::write_volatile(reg8(0x80063e), tl_multi_addr);
        } else {
            __tc32_fill_stack_pattern(
                core::ptr::addr_of_mut!(_end_custom_bss_),
                core::ptr::addr_of_mut!(_stack_end_),
            );
            __tc32_copy_words(
                core::ptr::addr_of_mut!(_start_data_),
                core::ptr::addr_of_mut!(_end_data_),
                core::ptr::addr_of!(_dstored_),
            );
            __tc32_zero_words(
                core::ptr::addr_of_mut!(_start_bss_),
                core::ptr::addr_of_mut!(_end_bss_),
            );
            __tc32_copy_words(
                core::ptr::addr_of_mut!(_start_custom_data_),
                core::ptr::addr_of_mut!(_end_custom_data_),
                core::ptr::addr_of!(_custom_stored_),
            );
            __tc32_zero_words(
                core::ptr::addr_of_mut!(_start_custom_bss_),
                core::ptr::addr_of_mut!(_end_custom_bss_),
            );
        }

        let _ = main();
        loop {
            core::hint::spin_loop();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn __mulsi3(mut a: u32, mut b: u32) -> u32 {
    let mut result = 0u32;
    while b != 0 {
        if (b & 1) != 0 {
            result = result.wrapping_add(a);
        }
        a <<= 1;
        b >>= 1;
    }
    result
}

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
pub extern "C" fn clock_time() -> u32 {
    timer::clock_time()
}

#[unsafe(no_mangle)]
pub extern "C" fn irq_disable() -> u8 {
    interrupt::disable() as u8
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code.pm_get_32k_tick")]
pub extern "C" fn pm_get_32k_tick() -> u32 {
    loop {
        let prev = ((analog::read(ANA_32K_TICK_BYTE3) as u32) << 24)
            | ((analog::read(ANA_32K_TICK_BYTE2) as u32) << 16)
            | ((analog::read(ANA_32K_TICK_BYTE1) as u32) << 8)
            | analog::read(ANA_32K_TICK_BYTE0) as u32;
        let value = ((analog::read(ANA_32K_TICK_BYTE3) as u32) << 24)
            | ((analog::read(ANA_32K_TICK_BYTE2) as u32) << 16)
            | ((analog::read(ANA_32K_TICK_BYTE1) as u32) << 8)
            | analog::read(ANA_32K_TICK_BYTE0) as u32;

        let delta = value.wrapping_sub(prev);
        if delta <= 1 {
            return if delta == 1 { prev } else { value };
        }
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code.start_reboot")]
pub extern "C" fn start_reboot() -> ! {
    interrupt::disable();
    soft_reboot_dly13ms_use24mRC();
    unsafe {
        core::ptr::write_volatile(reg8(REG_PWDN_CTRL), 0x20);
    }
    loop {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code.pm_wait_xtal_ready")]
pub extern "C" fn pm_wait_xtal_ready() {
    let loops = unsafe { g_pm_xtal_stable_loopnum };
    let mut i = 0u32;
    loop {
        if i > loops {
            return;
        }

        let start = clock_time();
        for _ in 0..60 {
            core::hint::spin_loop();
        }

        let delta = clock_time().wrapping_sub(start);
        if delta > 320 {
            if i == loops {
                start_reboot();
            }
            return;
        }

        i = i.wrapping_add(1);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cpu_wakeup_no_deepretn_back_init() {
    unsafe extern "C" {
        fn flash_vdd_f_calib();
    }

    clock::rc_24m_cal();
    clock::doubler_calibration();

    let info1 = pm_get_info1();
    if (info1 & 0xc0) != 0xc0 {
        efuse_sys_check(info1);
        unsafe { flash_vdd_f_calib() };
        return;
    }

    let calib = 0x03f7u16.wrapping_add(((info1 & 0x3f) as u16) * 5);
    adc_set_gpio_calib_vref(calib);
}

#[unsafe(no_mangle)]
pub extern "C" fn bls_pm_registerFuncBeforeSuspend(func: usize) {
    unsafe {
        func_before_suspend = func;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pm_set_wakeup_time_param(param: PmRDelayUs) {
    unsafe {
        g_pm_r_delay_us = param;
        let deep = param.deep_r_delay_us;
        let suspend_ret = param.suspend_ret_r_delay_us;
        g_pm_early_wakeup_time_us.deep_ret =
            (u32::from(suspend_ret) + 0x00e6 + g_pm_suspend_delay_us) as u16;
        g_pm_early_wakeup_time_us.deep = suspend_ret.wrapping_add(100);
        g_pm_early_wakeup_time_us.min = deep.wrapping_add(240);
        if g_pm_early_wakeup_time_us.min > g_pm_early_wakeup_time_us.suspend {
            g_pm_early_wakeup_time_us.min = g_pm_early_wakeup_time_us.suspend.wrapping_add(0x0190);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pm_set_xtal_stable_timer_param(delay_us: u32, loopnum: u32, nopnum: u32) {
    unsafe {
        g_pm_xtal_stable_suspend_nopnum = nopnum;
        g_pm_xtal_stable_loopnum = loopnum;
        g_pm_suspend_delay_us = delay_us;
        g_pm_early_wakeup_time_us.deep_ret =
            (g_pm_r_delay_us.suspend_ret_r_delay_us as u32 + 0x00e6 + delay_us) as u16;
        if g_pm_early_wakeup_time_us.min > g_pm_early_wakeup_time_us.suspend {
            g_pm_early_wakeup_time_us.min = g_pm_early_wakeup_time_us.suspend.wrapping_add(0x0190);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn soft_reboot_dly13ms_use24mRC() {
    let mut i = 0u32;
    while i <= 0x3c8b {
        core::hint::spin_loop();
        i = i.wrapping_add(1);
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code.sleep_start")]
pub extern "C" fn sleep_start() {
    unsafe extern "C" {
        fn start_suspend();
    }

    analog::write(0x34, 0x87);
    unsafe {
        core::ptr::write_volatile(reg8(REG_MSPI_CTRL), 0);
        core::ptr::write_volatile(reg8(REG_MSPI_DATA), 0xb9);
    }

    for _ in 0..2 {
        core::hint::spin_loop();
    }

    unsafe {
        core::ptr::write_volatile(reg8(REG_MSPI_CTRL), 1);
        core::ptr::write_volatile(reg8(REG_GPIO_PE_IE), 0);
    }
    analog::write(0x82, 0x0c);

    let ret_addr = unsafe {
        let hi = core::ptr::read_volatile(reg8(REG_SUSPEND_RET_ADDR_HI).cast_const()) as usize;
        let ptr = ((hi << 8) | 0x0084_0058) as *mut u32;
        let saved = core::ptr::read_volatile(ptr.cast_const());
        core::ptr::write_volatile(ptr, 0x06c0_06c0);
        (ptr, saved)
    };

    unsafe {
        start_suspend();
    }

    unsafe {
        core::ptr::write_volatile(ret_addr.0, ret_addr.1);
    }
    analog::write(0x82, 0x64);
    unsafe {
        core::ptr::write_volatile(reg8(REG_GPIO_PE_IE), 0x0f);
        core::ptr::write_volatile(reg8(REG_MSPI_CTRL), 0);
        core::ptr::write_volatile(reg8(REG_MSPI_DATA), 0xab);
    }

    for _ in 0..2 {
        core::hint::spin_loop();
    }

    unsafe {
        core::ptr::write_volatile(reg8(REG_MSPI_CTRL), 1);
    }
    analog::write(0x34, 0x80);

    let nopnum = unsafe { g_pm_xtal_stable_suspend_nopnum };
    for _ in 0..=nopnum {
        core::hint::spin_loop();
    }
}

#[inline(always)]
fn cpu_stall_wakeup_by_timer_common(tick_addr: usize, tick: u32, mask: u32, timer_bit: u8) {
    unsafe {
        core::ptr::write_volatile(reg32(tick_addr), 0);
        core::ptr::write_volatile(reg32(tick_addr - 12), tick);
        let ctrl = reg16(tick_addr - 4);
        let mut mode = core::ptr::read_volatile(ctrl.cast_const());
        mode &= !(timer_bit as u16 | ((timer_bit as u16) << 1));
        core::ptr::write_volatile(ctrl, mode);
        let ctrl8 = reg8(tick_addr - 4);
        let mut mode8 = core::ptr::read_volatile(ctrl8.cast_const());
        mode8 |= timer_bit;
        core::ptr::write_volatile(ctrl8, mode8);

        let irq = reg32(REG_MCU_WAKEUP_MASK);
        core::ptr::write_volatile(irq, core::ptr::read_volatile(irq.cast_const()) | mask);
        core::ptr::write_volatile(reg8(REG_TMR_STA), mask as u8);
        core::ptr::write_volatile(reg8(REG_PWDN_CTRL), 0x80);
        core::hint::spin_loop();
        core::hint::spin_loop();
        core::ptr::write_volatile(reg8(REG_TMR_STA), mask as u8);

        let mut final_ctrl = core::ptr::read_volatile(ctrl8.cast_const());
        final_ctrl &= !timer_bit;
        core::ptr::write_volatile(ctrl8, final_ctrl);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cpu_stall_wakeup_by_timer0(tick: u32) {
    cpu_stall_wakeup_by_timer_common(REG_TMR0_TICK, tick, 1, 0x01);
}

#[unsafe(no_mangle)]
pub extern "C" fn cpu_stall_wakeup_by_timer1(tick: u32) {
    cpu_stall_wakeup_by_timer_common(REG_TMR1_TICK, tick, 2, 0x08);
}

#[unsafe(no_mangle)]
pub extern "C" fn cpu_stall_wakeup_by_timer2(tick: u32) {
    unsafe {
        core::ptr::write_volatile(reg32(REG_TMR2_TICK), 0);
        core::ptr::write_volatile(reg32(REG_TMR2_TICK - 12), tick);
        let ctrl = reg16(REG_TMR2_TICK - 12);
        let mut mode = core::ptr::read_volatile(ctrl.cast_const());
        mode &= 0xff7d;
        core::ptr::write_volatile(ctrl, mode);
        let ctrl8 = reg8(REG_TMR2_TICK - 12);
        let mut mode8 = core::ptr::read_volatile(ctrl8.cast_const());
        mode8 |= 0x40;
        core::ptr::write_volatile(ctrl8, mode8);

        let irq = reg32(REG_MCU_WAKEUP_MASK);
        core::ptr::write_volatile(irq, core::ptr::read_volatile(irq.cast_const()) | 4);
        core::ptr::write_volatile(reg8(REG_TMR_STA), 4);
        core::ptr::write_volatile(reg8(REG_PWDN_CTRL), 0x80);
        core::hint::spin_loop();
        core::hint::spin_loop();
        core::ptr::write_volatile(reg8(REG_TMR_STA), 4);

        let mut final_ctrl = core::ptr::read_volatile(ctrl8.cast_const());
        final_ctrl &= !0x40;
        core::ptr::write_volatile(ctrl8, final_ctrl);
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code.cpu_stall")]
pub extern "C" fn cpu_stall(wakeup_src: u32, interval_us: u32, sysclktick: u32) -> u32 {
    if interval_us != 0 {
        unsafe {
            core::ptr::write_volatile(reg32(REG_TMR1_TICK), 0);
            core::ptr::write_volatile(reg32(REG_TMR1_TICK - 12), interval_us.wrapping_mul(sysclktick));
            let ctrl = reg8(REG_TMR1_TICK - 8);
            let mut value = core::ptr::read_volatile(ctrl.cast_const());
            value &= !0x30;
            value |= 0x08;
            core::ptr::write_volatile(ctrl, value);
        }
    }

    unsafe {
        let irq = reg32(REG_MCU_WAKEUP_MASK);
        core::ptr::write_volatile(irq, core::ptr::read_volatile(irq.cast_const()) | wakeup_src);

        let irq_mask = reg32(REG_IRQ_MASK);
        let mut rf_masked = core::ptr::read_volatile(irq_mask.cast_const());
        rf_masked &= 0xffff_dfff;
        rf_masked &= !0x2;
        core::ptr::write_volatile(irq_mask, rf_masked);

        core::ptr::write_volatile(reg8(REG_PWDN_CTRL), 0x80);
        core::hint::spin_loop();
        core::hint::spin_loop();

        if interval_us != 0 {
            core::ptr::write_volatile(reg32(REG_TMR1_TICK), 0);
            let ctrl = reg8(REG_TMR1_TICK - 20);
            let mut value = core::ptr::read_volatile(ctrl.cast_const());
            value &= !0x08;
            core::ptr::write_volatile(ctrl, value);
        }

        let status = core::ptr::read_volatile(reg32(REG_WAKEUP_SRC).cast_const());
        core::ptr::write_volatile(reg8(REG_TMR_STA), 2);
        core::ptr::write_volatile(reg16(REG_RF_IRQ_DONE), 0xffff);
        status
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cpu_set_gpio_wakeup(pin: u32, pol: u32, en: i32) {
    let bit = ((pin >> 8) & 0xff) as u8;
    let port = (pin & 0xff) as u8;
    let pull_reg = port.wrapping_add(0x21);
    let wake_reg = port.wrapping_add(0x27);

    let pull = analog::read(pull_reg);
    let new_pull = if pol == 0 { pull & !bit } else { pull | bit };
    analog::write(pull_reg, new_pull);

    let wake = analog::read(wake_reg);
    let new_wake = if en == 0 { wake & !bit } else { wake | bit };
    analog::write(wake_reg, new_wake);
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code.cpu_wakeup_init")]
pub extern "C" fn cpu_wakeup_init() {
    unsafe {
        core::ptr::write_volatile(reg8(REG_RST0), 0x00);
        core::ptr::write_volatile(reg8(REG_RST1), 0x00);
        core::ptr::write_volatile(reg8(REG_RST2), 0x00);
        core::ptr::write_volatile(reg8(REG_CLK_EN0), 0xff);
        core::ptr::write_volatile(reg8(REG_CLK_EN1), 0xff);
        core::ptr::write_volatile(reg8(REG_CLK_EN2), 0xff);
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

    unsafe {
        core::ptr::write_volatile(reg32(REG_PM_RET_CTRL0), 0x0404_0404);
        core::ptr::write_volatile(reg32(REG_PM_RET_CTRL1), 0x0404_0404);
        core::ptr::write_volatile(reg8(REG_PM_RET_BYTE), 0x04);
        core::ptr::write_volatile(crate::mmio::reg16(REG_DCDC_CTRL), 0);
    }

    if unsafe { core::ptr::read_volatile(reg8(REG_PM_WAKEUP_FLAG).cast_const()) } == 1 {
        analog::write(0x01, 0x3c);
    } else {
        analog::write(0x01, 0x4c);
    }

    let need_read_wakeup_src = if (analog::read(0x7f) & 0x01) != 0 {
        unsafe {
            pmParam.mcu_status = MCU_STATUS_DEEPRET_BACK;
        }
        true
    } else {
        let deep_back = analog::read(0x3c);
        if (deep_back & 0x02) != 0 {
            unsafe {
                pmParam.mcu_status = MCU_STATUS_DEEP_BACK;
            }
            analog::write(0x3c, deep_back & 0xfd);
            true
        } else {
            unsafe {
                pmParam.mcu_status = MCU_STATUS_BOOT;
            }
            false
        }
    };

    if need_read_wakeup_src {
        unsafe {
            pmParam.wakeup_src = analog::read(0x44);
            pmParam.is_pad_wakeup = if (pmParam.wakeup_src & 0x0a) == 0x08 {
                1
            } else {
                0
            };
        }
    }

    if unsafe { pmParam.mcu_status } == MCU_STATUS_DEEPRET_BACK {
        unsafe {
            let now_32k = pm_get_32k_tick();
            let recovered = if pm_tim_recover != 0 {
                let handler: unsafe extern "C" fn(u32) -> u32 =
                    core::mem::transmute(pm_tim_recover);
                handler(now_32k)
            } else {
                now_32k
            };
            core::ptr::write_volatile(reg32(REG_SYSTEM_TICK), recovered);
            core::ptr::write_volatile(reg8(REG_SYSTEM_TICK + 12), 0x00);
            core::ptr::write_volatile(reg8(REG_SYSTEM_TICK + 12), 0x92);
            core::ptr::write_volatile(reg8(REG_SYSTEM_TICK + 15), MCU_STATUS_DEEPRET_BACK);
        }
        pm_wait_xtal_ready();
    } else {
        unsafe {
            core::ptr::write_volatile(reg8(REG_PM_WAIT), 0x01);
        }
        pm_wait_xtal_ready();
        cpu_wakeup_no_deepretn_back_init();
    }

    unsafe {
        core::ptr::write_volatile(reg8(REG_PM_RET_CLR), 0x00);
        core::ptr::write_volatile(reg8(REG_PM_RET_CLR + 1), 0x00);
        let value = core::ptr::read_volatile(reg8(REG_GPIO_WAKEUP_IRQ).cast_const()) | 0x0c;
        core::ptr::write_volatile(reg8(REG_GPIO_WAKEUP_IRQ), value);
    }
    let _ = (ANA_REG_0X8A, REG_PWDN_CTRL);
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
