use crate::mmio::{reg16, reg32, reg8};
#[cfg(feature = "chip-8258")]
use crate::regs8258::{
    ANA_32K_TICK_BYTE0, ANA_32K_TICK_BYTE1, ANA_32K_TICK_BYTE2, ANA_32K_TICK_BYTE3, ANA_REG_0X02,
    ANA_REG_0X27, ANA_REG_0X28, ANA_REG_0X29, ANA_REG_0X2A, ANA_REG_0X8A, ANA_REG_0X8C,
    ANA_USB_DP_PULLUP, ANA_USB_POWER, AREG_CLK_SETTING, REG_ANA_POWER_CTRL, REG_CLK_EN0,
    REG_CLK_EN1, REG_CLK_EN2, REG_DCDC_CTRL, REG_DFIFO0_ADDR, REG_DFIFO0_SIZE, REG_DFIFO1_ADDR,
    REG_DMA_CHN_EN, REG_GPIO_PE_IE, REG_GPIO_WAKEUP_IRQ, REG_IRQ_MASK, REG_MCU_WAKEUP_MASK,
    REG_MSPI_CTRL, REG_MSPI_DATA, REG_PM_INFO0, REG_PM_INFO1, REG_PM_WAKEUP_FLAG, REG_PWDN_CTRL,
    REG_RF_IRQ_STATUS, REG_RST0, REG_RST1, REG_RST2, REG_SUSPEND_RET_ADDR_HI, REG_SYSTEM_TICK,
    REG_SYSTEM_TICK_CTRL, REG_TMR0_TICK, REG_TMR1_TICK, REG_TMR2_TICK, REG_TMR_STA, REG_WAKEUP_SRC,
    REG_CLK_SEL,
};
use crate::{analog, clock, gpio, interrupt, timer};

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

#[repr(C, align(4))]
pub struct MiscPara {
    pub ext_cap_en: u8,
    pub pad32k_en: u8,
    pub pm_enter_en: u8,
    pub _reserved: u8,
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
const TCMD_UNDER_WR: u8 = 0x40;
const TCMD_MASK: u8 = 0x3f;
const TCMD_WRITE: u8 = 0x03;
const TCMD_WAIT: u8 = 0x07;
const TCMD_WAREG: u8 = 0x08;

#[repr(C)]
pub struct TblCmdSet {
    pub adr: u16,
    pub dat: u8,
    pub cmd: u8,
}

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
pub static mut pm_check_32k_clk_stable: usize = 0;

#[unsafe(no_mangle)]
pub static mut tl_multi_addr: u8 = 0;

#[unsafe(no_mangle)]
pub static mut blt_miscParam: MiscPara = MiscPara {
    ext_cap_en: 0,
    pad32k_en: 0,
    pm_enter_en: 0,
    _reserved: 0,
};

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
pub static mut PM_STARTUP_DBG_WAKEUP_FLAG: u8 = 0;
#[unsafe(no_mangle)]
pub static mut PM_STARTUP_DBG_ANA7F: u8 = 0;
#[unsafe(no_mangle)]
pub static mut PM_STARTUP_DBG_ANA3C: u8 = 0;
#[unsafe(no_mangle)]
pub static mut PM_STARTUP_DBG_STAGE: u8 = 0;
#[unsafe(no_mangle)]
pub static mut PM_STARTUP_DBG_SUBSTAGE: u8 = 0;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".custom_bss.pm_startup_dbg_magic")]
pub static mut PM_PERSIST_DBG_MAGIC: u32 = 0;
#[unsafe(no_mangle)]
#[unsafe(link_section = ".custom_bss.pm_startup_dbg_stage")]
pub static mut PM_PERSIST_DBG_STAGE: u8 = 0;
#[unsafe(no_mangle)]
#[unsafe(link_section = ".custom_bss.pm_startup_dbg_substage")]
pub static mut PM_PERSIST_DBG_SUBSTAGE: u8 = 0;
#[unsafe(no_mangle)]
#[unsafe(link_section = ".custom_bss.pm_startup_dbg_wakeup_flag")]
pub static mut PM_PERSIST_DBG_WAKEUP_FLAG: u8 = 0;
#[unsafe(no_mangle)]
#[unsafe(link_section = ".custom_bss.pm_startup_dbg_ana7f")]
pub static mut PM_PERSIST_DBG_ANA7F: u8 = 0;
#[unsafe(no_mangle)]
#[unsafe(link_section = ".custom_bss.pm_startup_dbg_ana3c")]
pub static mut PM_PERSIST_DBG_ANA3C: u8 = 0;

#[inline(always)]
fn persist_startup_stage(stage: u8, substage: u8) {
    unsafe {
        core::ptr::write_volatile(&raw mut PM_STARTUP_DBG_STAGE, stage);
        core::ptr::write_volatile(&raw mut PM_STARTUP_DBG_SUBSTAGE, substage);
        core::ptr::write_volatile(&raw mut PM_PERSIST_DBG_MAGIC, 0x504d_4442);
        core::ptr::write_volatile(&raw mut PM_PERSIST_DBG_STAGE, stage);
        core::ptr::write_volatile(&raw mut PM_PERSIST_DBG_SUBSTAGE, substage);
    }
}

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
pub unsafe extern "C" fn __tc32_copy_words(mut dst: *mut u32, end: *mut u32, mut src: *const u32) {
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
        let value = core::ptr::read_volatile(ana.add(1).cast_const());
        // Vendor parity: analog_read() always clears reg_ana_ctrl after the
        // transaction. Leaving 0x40 latched here breaks subsequent early-boot
        // analog writes/reads, including PM wake markers in __tc32_boot_init().
        core::ptr::write_volatile(ana.add(2), 0);
        value
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn __tc32_analog_write_u8(reg: u8, value: u8) {
    unsafe {
        let ana = reg8(0x8000b8);
        core::ptr::write_volatile(ana, reg);
        core::ptr::write_volatile(ana.add(1), value);
        core::ptr::write_volatile(ana.add(2), 0x60);
        while (core::ptr::read_volatile(ana.add(2).cast_const()) & 1) != 0 {}
        core::ptr::write_volatile(ana.add(2), 0);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn __tc32_system_on_for_flash() {
    unsafe {
        // Keep the flash power/clock bring-up byte-for-byte aligned with the
        // official 8258 startup path. Diverging values here left flash asleep
        // after retention wake, and boot stalled before the first fetch in
        // `main()`.
        core::ptr::write_volatile(reg32(0x800060), 0xff00_0000);
        core::ptr::write_volatile(reg8(0x800064), 0xff);
        core::ptr::write_volatile(reg8(0x800065), 0xff);
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
        // Vendor boot code uses a volatile 0..=6 loop here. A plain Rust
        // `spin_loop()` loop was optimized away completely, which collapsed the
        // flash wake sequence to back-to-back writes and left flash asleep
        // after retention wake.
        let mut delay = 0u32;
        while unsafe { core::ptr::read_volatile(&raw const delay) } <= 6 {
            let next = unsafe { core::ptr::read_volatile(&raw const delay) }.wrapping_add(1);
            unsafe { core::ptr::write_volatile(&raw mut delay, next) };
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

pub fn startup_check_32k_clk_stable() {
    startup_pm_wait_xtal_ready();
}

#[unsafe(no_mangle)]
pub extern "C" fn LoadTblCmdSet(pt: *const TblCmdSet, size: i32) -> i32 {
    if pt.is_null() || size <= 0 {
        return 0;
    }

    let mut i = 0i32;
    while i < size {
        let entry = unsafe { core::ptr::read_volatile(pt.add(i as usize)) };
        let cmd_raw = entry.cmd;
        if (cmd_raw & TCMD_UNDER_WR) != 0 {
            match cmd_raw & TCMD_MASK {
                TCMD_WRITE => unsafe {
                    core::ptr::write_volatile(reg8(0x0080_0000 | (entry.adr as usize)), entry.dat)
                },
                TCMD_WAREG => analog::write(entry.adr as u8, entry.dat),
                TCMD_WAIT => {
                    let delay_us = ((entry.adr as u32) << 8) | (entry.dat as u32);
                    let t0 = timer::clock_time();
                    while !timer::clock_time_exceed_us(t0, delay_us) {
                        core::hint::spin_loop();
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    size
}

#[inline(always)]
fn boot_init_pa7_sws() {
    // Force PA7 to SWire and input-enabled as early as possible.
    // This keeps SWD/SWS attach reliable even if PM flow fails later.
    unsafe {
        let mux_pa_4_7 = reg8(0x0080_05a9);
        let mux = core::ptr::read_volatile(mux_pa_4_7.cast_const()) & !(0b11 << 6);
        core::ptr::write_volatile(mux_pa_4_7, mux);

        let gpio_func = reg8(0x0080_0586);
        let func = core::ptr::read_volatile(gpio_func.cast_const()) & !0x80;
        core::ptr::write_volatile(gpio_func, func);

        let gpio_ie = reg8(0x0080_0581);
        let ie = core::ptr::read_volatile(gpio_ie.cast_const()) | 0x80;
        core::ptr::write_volatile(gpio_ie, ie);

        let gpio_oen = reg8(0x0080_0582);
        let oen = core::ptr::read_volatile(gpio_oen.cast_const()) | 0x80;
        core::ptr::write_volatile(gpio_oen, oen);
    }
}

#[inline(always)]
fn boot_init_has_saved_pm_mode(wake_flag: u8, wake_status: u8, pm_status: u8) -> bool {
    if !matches!(wake_flag, 0x80 | 0x61 | 0x43 | 0x07 | 0xff) {
        return false;
    }

    // VENDOR-DIFF:
    // `ana 0x7e` can remain stale after external reset/activation, which makes
    // boot init skip `.data/.bss` setup and leaves retained-RAM state random.
    // Treat saved PM mode as valid only when there is also a real wake
    // indicator: deep wake flag in `ana 0x7f[0]` or a wake source latched in
    // `ana 0x44[3:0]`.
    (pm_status & 0x01) != 0 || (wake_status & 0x0f) != 0
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".vectors.boot")]
pub extern "C" fn __tc32_boot_init() -> ! {
    unsafe {
        boot_init_pa7_sws();
        __tc32_init_icache(
            core::ptr::addr_of_mut!(_ictag_start_),
            core::ptr::addr_of_mut!(_ictag_end_),
            core::ptr::addr_of_mut!(_ramcode_size_align_256_),
        );
        __tc32_system_on_for_flash();
        __tc32_flash_wakeup();
        __tc32_efuse_delay();

        let wake_flag = __tc32_analog_read_u8(0x7e);
        let wake_status = __tc32_analog_read_u8(0x44);
        let pm_status = __tc32_analog_read_u8(0x7f);
        // VENDOR-DIFF:
        // vendor boot code treats any non-zero `ana 0x7e` as a PM wake marker.
        // Keep the check restricted to vendor PM mode encodings to reject stale
        // garbage after hard reset/activation, but do not additionally require
        // `ana 0x44[3:0] != 0`: that extra gate can misclassify a real
        // retention/deep wake as cold boot and break the wake path.
        //
        // Also keep boot init off analog regs `0x3a..0x3f`. `pmled8258` uses
        // `0x3a` for persisted testcase state, and clobbering it here makes the
        // example restart from testcase 1 after each wake.
        if boot_init_has_saved_pm_mode(wake_flag, wake_status, pm_status) {
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
#[unsafe(link_section = ".text.rust_entry")]
pub extern "C" fn __tc32_rust_entry() -> ! {
    let _ = unsafe { main() };
    loop {
        core::hint::spin_loop();
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

pub fn startup_pm_get_32k_tick() -> u32 {
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

pub fn startup_start_reboot() -> ! {
    interrupt::disable();
    startup_soft_reboot_dly13ms_use24m_rc();
    unsafe {
        core::ptr::write_volatile(reg8(REG_PWDN_CTRL), 0x20);
    }
    loop {
        core::hint::spin_loop();
    }
}

pub fn startup_pm_wait_xtal_ready() {
    let loops = unsafe { core::ptr::read_volatile(&raw const g_pm_xtal_stable_loopnum) };
    let mut i = 0u32;
    while i < loops {
        let start = clock_time();

        let mut j = 0u32;
        // Vendor uses a volatile local delay counter here; keep it volatile to
        // preserve the wait window before checking `clock_time_exceed(start, 20)`.
        while unsafe { core::ptr::read_volatile(&raw const j) } <= 0x3b {
            let next = unsafe { core::ptr::read_volatile(&raw const j) }.wrapping_add(1);
            unsafe { core::ptr::write_volatile(&raw mut j, next) };
        }

        if timer::clock_time_exceed_us(start, 20) {
            return;
        }

        i = i.wrapping_add(1);
    }

    startup_start_reboot();
}

#[unsafe(no_mangle)]
pub extern "C" fn cpu_wakeup_no_deepretn_back_init() {
    unsafe extern "C" {
        fn flash_vdd_f_calib();
    }

    persist_startup_stage(0x20, 0x41);
    clock::rc_24m_cal();
    persist_startup_stage(0x20, 0x42);
    clock::doubler_calibration();
    persist_startup_stage(0x20, 0x43);

    let info1 = pm_get_info1();
    if (info1 & 0xc0) != 0xc0 {
        persist_startup_stage(0x20, 0x44);
        efuse_sys_check(info1);
        persist_startup_stage(0x20, 0x45);
        unsafe { flash_vdd_f_calib() };
        persist_startup_stage(0x20, 0x46);
        return;
    }

    let calib = 0x03f7u16.wrapping_add(((info1 & 0x3f) as u16) * 5);
    persist_startup_stage(0x20, 0x47);
    adc_set_gpio_calib_vref(calib);
    persist_startup_stage(0x20, 0x48);
}

pub fn startup_bls_pm_register_func_before_suspend(func: usize) {
    unsafe {
        func_before_suspend = func;
    }
}

pub fn startup_pm_set_wakeup_time_param(param: PmRDelayUs) {
    unsafe {
        g_pm_r_delay_us = param;
        let deep = param.deep_r_delay_us;
        let suspend_ret = param.suspend_ret_r_delay_us;
        g_pm_early_wakeup_time_us.suspend =
            (u32::from(suspend_ret) + 0x00e6 + g_pm_suspend_delay_us) as u16;
        g_pm_early_wakeup_time_us.deep_ret = suspend_ret.wrapping_add(100);
        g_pm_early_wakeup_time_us.deep = deep.wrapping_add(240);
        if g_pm_early_wakeup_time_us.deep < g_pm_early_wakeup_time_us.suspend {
            g_pm_early_wakeup_time_us.min = g_pm_early_wakeup_time_us.deep.wrapping_add(0x0190);
        } else {
            g_pm_early_wakeup_time_us.min = g_pm_early_wakeup_time_us.suspend.wrapping_add(0x0190);
        }
    }
}

pub fn startup_pm_set_xtal_stable_timer_param(delay_us: u32, loopnum: u32, nopnum: u32) {
    unsafe {
        g_pm_xtal_stable_suspend_nopnum = nopnum;
        g_pm_xtal_stable_loopnum = loopnum;
        g_pm_suspend_delay_us = delay_us;
        g_pm_early_wakeup_time_us.suspend =
            (g_pm_r_delay_us.suspend_ret_r_delay_us as u32 + 0x00e6 + delay_us) as u16;
        if g_pm_early_wakeup_time_us.deep < g_pm_early_wakeup_time_us.suspend {
            g_pm_early_wakeup_time_us.min = g_pm_early_wakeup_time_us.deep.wrapping_add(0x0190);
        } else {
            g_pm_early_wakeup_time_us.min = g_pm_early_wakeup_time_us.suspend.wrapping_add(0x0190);
        }
    }
}

pub fn startup_soft_reboot_dly13ms_use24m_rc() {
    let mut i = 0u32;
    while i <= 0x3c8b {
        core::hint::spin_loop();
        i = i.wrapping_add(1);
    }
}

#[unsafe(link_section = ".ram_code.startup_sleep_start")]
pub fn startup_sleep_start() {
    unsafe extern "C" {
        fn start_suspend();
    }

    analog::write(0x34, 0x87);
    unsafe {
        core::ptr::write_volatile(reg8(REG_MSPI_CTRL), 0);
        core::ptr::write_volatile(reg8(REG_MSPI_DATA), 0xb9);
    }

    let mut delay = 0u32;
    while unsafe { core::ptr::read_volatile(&raw const delay) } <= 1 {
        let next = unsafe { core::ptr::read_volatile(&raw const delay) }.wrapping_add(1);
        unsafe { core::ptr::write_volatile(&raw mut delay, next) };
    }

    unsafe {
        core::ptr::write_volatile(reg8(REG_MSPI_CTRL), 1);
        core::ptr::write_volatile(reg8(REG_GPIO_PE_IE), 0);
    }
    analog::write(AREG_CLK_SETTING, 0x0c);

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
    analog::write(AREG_CLK_SETTING, 0x64);
    unsafe {
        core::ptr::write_volatile(reg8(REG_GPIO_PE_IE), 0x0f);
        core::ptr::write_volatile(reg8(REG_MSPI_CTRL), 0);
        core::ptr::write_volatile(reg8(REG_MSPI_DATA), 0xab);
    }

    let mut delay = 0u32;
    while unsafe { core::ptr::read_volatile(&raw const delay) } <= 1 {
        let next = unsafe { core::ptr::read_volatile(&raw const delay) }.wrapping_add(1);
        unsafe { core::ptr::write_volatile(&raw mut delay, next) };
    }

    unsafe {
        core::ptr::write_volatile(reg8(REG_MSPI_CTRL), 1);
    }
    analog::write(0x34, 0x80);

    let nopnum = unsafe { g_pm_xtal_stable_suspend_nopnum };
    let mut delay = 0u32;
    while unsafe { core::ptr::read_volatile(&raw const delay) } <= nopnum {
        let next = unsafe { core::ptr::read_volatile(&raw const delay) }.wrapping_add(1);
        unsafe { core::ptr::write_volatile(&raw mut delay, next) };
    }
}

#[inline(always)]
fn cpu_stall_wakeup_by_timer_common(
    tick_addr: usize,
    tick: u32,
    mask: u32,
    timer_bit: u8,
    mode_clear_mask: u16,
    ctrl_offset_from_tick: usize,
) {
    unsafe {
        core::ptr::write_volatile(reg32(tick_addr), 0);
        core::ptr::write_volatile(reg32(tick_addr - 12), tick);
        let ctrl_addr = tick_addr - ctrl_offset_from_tick;
        let ctrl = reg16(ctrl_addr);
        let mut mode = core::ptr::read_volatile(ctrl.cast_const());
        mode &= !mode_clear_mask;
        core::ptr::write_volatile(ctrl, mode);
        let ctrl8 = reg8(ctrl_addr);
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

pub fn startup_cpu_stall_wakeup_by_timer0(tick: u32) {
    cpu_stall_wakeup_by_timer_common(REG_TMR0_TICK, tick, 1, 0x01, 0x06, 16);
}

pub fn startup_cpu_stall_wakeup_by_timer1(tick: u32) {
    cpu_stall_wakeup_by_timer_common(REG_TMR1_TICK, tick, 2, 0x08, 0x30, 20);
}

pub fn startup_cpu_stall_wakeup_by_timer2(tick: u32) {
    cpu_stall_wakeup_by_timer_common(REG_TMR2_TICK, tick, 4, 0x40, 0x0082, 24);
}

pub fn startup_cpu_stall(wakeup_src: u32, interval_us: u32, sysclktick: u32) -> u32 {
    if interval_us != 0 {
        unsafe {
            core::ptr::write_volatile(reg32(REG_TMR1_TICK), 0);
            core::ptr::write_volatile(
                reg32(REG_TMR1_TICK - 12),
                interval_us.wrapping_mul(sysclktick),
            );
            core::ptr::write_volatile(reg8(REG_TMR_STA), 2);
            let ctrl = reg8(REG_TMR1_TICK - 20);
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

pub fn startup_cpu_set_gpio_wakeup(pin: u32, pol: u32, en: i32) {
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

pub fn startup_cpu_wakeup_init() {
    persist_startup_stage(0x10, 0x10);
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
    persist_startup_stage(0x11, 0x10);

    let sram_shutdown_sel =
        unsafe { core::ptr::read_volatile(reg8(REG_PM_WAKEUP_FLAG).cast_const()) };
    let ana_7f = analog::read(0x7f);
    let ana_3c = analog::read(0x3c);
    unsafe {
        PM_STARTUP_DBG_WAKEUP_FLAG = sram_shutdown_sel;
        PM_STARTUP_DBG_ANA7F = ana_7f;
        PM_STARTUP_DBG_ANA3C = ana_3c;
        PM_PERSIST_DBG_WAKEUP_FLAG = sram_shutdown_sel;
        PM_PERSIST_DBG_ANA7F = ana_7f;
        PM_PERSIST_DBG_ANA3C = ana_3c;
    }
    persist_startup_stage(0x12, 0x10);
    if sram_shutdown_sel == 1 {
        analog::write(0x01, 0x3c);
    } else {
        analog::write(0x01, 0x4c);
    }

    if (ana_7f & 0x01) != 0 {
        persist_startup_stage(0x20, 0x20);
        unsafe {
            pmParam.mcu_status = MCU_STATUS_DEEP_BACK;
        }
        analog::write(0x3c, ana_3c & 0xfd);
        unsafe {
            core::ptr::write_volatile(reg8(REG_SYSTEM_TICK_CTRL), 0x01);
        }
        persist_startup_stage(0x20, 0x22);
        startup_pm_wait_xtal_ready();
        persist_startup_stage(0x20, 0x23);
        cpu_wakeup_no_deepretn_back_init();
        persist_startup_stage(0x20, 0x24);
    } else {
        persist_startup_stage(0x30, 0x30);
        unsafe {
            pmParam.mcu_status = MCU_STATUS_DEEPRET_BACK;
        }
    }

    unsafe {
        pmParam.wakeup_src = analog::read(0x44);
        pmParam.is_pad_wakeup = if (pmParam.wakeup_src & 0x0a) == 0x08 {
            1
        } else {
            0
        };
    }

    if unsafe { pmParam.mcu_status } == MCU_STATUS_DEEPRET_BACK {
        persist_startup_stage(0x30, 0x31);
        unsafe {
            let now_32k = startup_pm_get_32k_tick();
            tick_cur = if pm_tim_recover != 0 {
                let handler: unsafe extern "C" fn(u32) -> u32 = core::mem::transmute(pm_tim_recover);
                handler(now_32k)
            } else {
                now_32k
            };
            core::ptr::write_volatile(reg8(REG_SYSTEM_TICK + 12), 0x00);
            core::ptr::write_volatile(reg8(REG_SYSTEM_TICK + 12), 0x92);
            core::ptr::write_volatile(reg8(REG_SYSTEM_TICK_CTRL), 0x01);
        }
        persist_startup_stage(0x30, 0x32);
        startup_pm_wait_xtal_ready();
        persist_startup_stage(0x30, 0x33);
    } else {
        // VENDOR-DIFF: keep explicit startup breadcrumbs, but preserve the
        // vendor's second deep-wake init pass and ordering.
        persist_startup_stage(0x20, 0x31);
        unsafe {
            core::ptr::write_volatile(reg8(REG_SYSTEM_TICK_CTRL), 0x01);
        }
        persist_startup_stage(0x20, 0x32);
        startup_pm_wait_xtal_ready();
        persist_startup_stage(0x20, 0x33);
        cpu_wakeup_no_deepretn_back_init();
        persist_startup_stage(0x20, 0x34);
    }

    unsafe {
        core::ptr::write_volatile(reg8(REG_PM_RET_CLR), 0x00);
        core::ptr::write_volatile(reg8(REG_PM_RET_CLR + 1), 0x00);
        let value = core::ptr::read_volatile(reg8(REG_GPIO_WAKEUP_IRQ).cast_const()) | 0x0c;
        core::ptr::write_volatile(reg8(REG_GPIO_WAKEUP_IRQ), value);
    }
    persist_startup_stage(0x40, 0x40);
    let _ = (ANA_REG_0X8A, REG_PWDN_CTRL);
}

#[inline(always)]
pub fn startup_state() -> StartupState {
    match unsafe { core::ptr::read_volatile(&raw const pmParam.mcu_status) } {
        MCU_STATUS_DEEPRET_BACK => StartupState::DeepRetention,
        MCU_STATUS_DEEP_BACK => StartupState::Deep,
        _ => StartupState::Boot,
    }
}

#[inline(always)]
pub fn wakeup_src_raw() -> u8 {
    unsafe { core::ptr::read_volatile(&raw const pmParam.wakeup_src) }
}

#[inline(always)]
pub fn is_pad_wakeup() -> bool {
    unsafe { core::ptr::read_volatile(&raw const pmParam.is_pad_wakeup) != 0 }
}

#[inline(always)]
pub fn set_pm_tim_recover_handler(handler: usize) {
    unsafe {
        core::ptr::write_volatile(&raw mut pm_tim_recover, handler);
    }
}

#[inline(always)]
pub fn set_cpu_sleep_wakeup_handler(handler: usize) {
    unsafe {
        core::ptr::write_volatile(&raw mut cpu_sleep_wakeup, handler);
    }
}

#[inline(always)]
pub fn set_pm_check_32k_clk_stable_handler(handler: usize) {
    unsafe {
        core::ptr::write_volatile(&raw mut pm_check_32k_clk_stable, handler);
    }
}

#[inline(always)]
pub fn set_misc_pad32k_enabled(enabled: bool) {
    unsafe {
        core::ptr::write_volatile(&raw mut blt_miscParam.pad32k_en, enabled as u8);
    }
}

#[inline(always)]
pub fn set_misc_pm_enter_enabled(enabled: bool) {
    unsafe {
        core::ptr::write_volatile(&raw mut blt_miscParam.pm_enter_en, enabled as u8);
    }
}

#[inline(always)]
pub fn set_tick_cur(value: u32) {
    unsafe {
        core::ptr::write_volatile(&raw mut tick_cur, value);
    }
}

#[inline(always)]
pub fn current_tick_cur() -> u32 {
    unsafe { core::ptr::read_volatile(&raw const tick_cur) }
}

#[inline(always)]
pub fn set_tick_32k_cur(value: u32) {
    unsafe {
        core::ptr::write_volatile(&raw mut tick_32k_cur, value);
    }
}

#[inline(always)]
pub fn set_tick_32k_calib(value: u16) {
    unsafe {
        core::ptr::write_volatile(&raw mut tick_32k_calib, value);
    }
}

#[inline(always)]
pub fn current_tick_32k_cur() -> u32 {
    unsafe { core::ptr::read_volatile(&raw const tick_32k_cur) }
}

#[inline(always)]
pub fn set_pm_long_suspend(value: bool) {
    unsafe {
        core::ptr::write_volatile(&raw mut pm_long_suspend, value as u8);
    }
}

#[inline(always)]
pub fn init() -> StartupState {
    persist_startup_stage(0x01, 0x01);
    interrupt::disable();
    interrupt::clear_mask(interrupt::ALL_IRQS);
    interrupt::clear_all_irq_sources();

    if let Ok(pa7) = gpio::RawPin::try_from_u16(0x0080) {
        let _ = gpio::set_function_for_raw_pin(pa7, gpio::PinFunction::Swire);
        gpio::set_input_enabled_raw(pa7, true);
    }

    persist_startup_stage(0x02, 0x01);
    crate::pm::cpu_wakeup_init();
    persist_startup_stage(0x03, 0x01);
    clock::init(clock::SysClock::Crystal48M);
    persist_startup_stage(0x04, 0x01);
    if startup_state() == StartupState::Boot {
        unsafe {
            core::ptr::write_volatile(reg32(REG_SYSTEM_TICK), 0);
            core::ptr::write_volatile(reg8(REG_SYSTEM_TICK + 12), 0x00);
            core::ptr::write_volatile(reg8(REG_SYSTEM_TICK + 12), 0x12);
            core::ptr::write_volatile(reg8(REG_SYSTEM_TICK_CTRL), 0x01);
        }
    }
    unsafe {
        sysTimerPerUs = timer::sys_tick_per_us();
    }
    persist_startup_stage(0x05, 0x01);

    startup_state()
}
