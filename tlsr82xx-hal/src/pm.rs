use crate::{analog, gpio, interrupt, startup, timer};

#[cfg(feature = "chip-8258")]
use crate::mmio::{reg16, reg32, reg8};
#[cfg(feature = "chip-8258")]
use crate::regs8258::{REG_MCU_WAKEUP_MASK, REG_PWDN_CTRL};

#[cfg(feature = "chip-8258")]
const REG_SYSTEM_WAKEUP_TICK: usize = 0x0080_0748;

const SYS_TICK_HZ: u32 = 16_000_000;
const RC_32K_HZ: u32 = 32_000;
const XTAL_32K_HZ: u32 = 32_768;

const PM_WAKEUP_PAD_BITS: u8 = 1 << 4;
const PM_WAKEUP_CORE_BITS: u8 = 1 << 5;
const PM_WAKEUP_TIMER_BITS: u8 = 1 << 6;
const PM_WAKEUP_COMPARATOR_BITS: u8 = 1 << 7;
const WAKEUP_STATUS_ALL: u8 = 0x0f;
const CRYSTAL32768_TICK_PER_32CYCLE: u32 = 15625;

pub const WAKEUP_STATUS_COMPARATOR: u32 = 1 << 0;
pub const WAKEUP_STATUS_TIMER: u32 = 1 << 1;
pub const WAKEUP_STATUS_CORE: u32 = 1 << 2;
pub const WAKEUP_STATUS_PAD: u32 = 1 << 3;
pub const WAKEUP_STATUS_WD: u32 = 1 << 6;
pub const STATUS_GPIO_ERR_NO_ENTER_PM: u32 = 1 << 8;
pub const STATUS_ENTER_SUSPEND: u32 = 1 << 30;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SleepMode {
    Suspend = 0x00,
    #[cfg(feature = "chip-8258")]
    DeepSleep = 0x80,
    #[cfg(feature = "chip-8258")]
    DeepSleepRetentionLow8K = 0x61,
    #[cfg(feature = "chip-8258")]
    DeepSleepRetentionLow16K = 0x43,
    #[cfg(feature = "chip-8258")]
    DeepSleepRetentionLow32K = 0x07,
    #[cfg(feature = "chip-8258")]
    Shutdown = 0xff,
    #[cfg(feature = "chip-8278")]
    DeepSleep = 0x30,
    #[cfg(feature = "chip-8278")]
    DeepSleepRetentionLow16K = 0x21,
    #[cfg(feature = "chip-8278")]
    DeepSleepRetentionLow32K = 0x03,
}

impl SleepMode {
    #[inline(always)]
    pub const fn raw(self) -> u8 {
        self as u8
    }

    #[inline(always)]
    pub const fn is_suspend(self) -> bool {
        matches!(self, Self::Suspend)
    }

    #[inline(always)]
    #[cfg(feature = "chip-8258")]
    pub const fn retains_sram(self) -> bool {
        matches!(
            self,
            Self::DeepSleepRetentionLow8K
                | Self::DeepSleepRetentionLow16K
                | Self::DeepSleepRetentionLow32K
        )
    }

    #[inline(always)]
    #[cfg(not(feature = "chip-8258"))]
    pub const fn retains_sram(self) -> bool {
        let _ = self;
        false
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct WakeupSource(u8);

impl WakeupSource {
    pub const NONE: Self = Self(0);
    pub const PAD: Self = Self(PM_WAKEUP_PAD_BITS);
    pub const CORE: Self = Self(PM_WAKEUP_CORE_BITS);
    pub const TIMER: Self = Self(PM_WAKEUP_TIMER_BITS);
    pub const COMPARATOR: Self = Self(PM_WAKEUP_COMPARATOR_BITS);

    #[inline(always)]
    pub const fn raw(self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl core::ops::BitOr for WakeupSource {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for WakeupSource {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Clock32kSource {
    InternalRc,
    ExternalCrystal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WakeOrigin {
    ColdBoot,
    DeepWake,
    DeepRetentionWake,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum WakeupLevel {
    Low = 0,
    High = 1,
}

impl From<gpio::Level> for WakeupLevel {
    fn from(value: gpio::Level) -> Self {
        match value {
            gpio::Level::Low => Self::Low,
            gpio::Level::High => Self::High,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WakeupTiming {
    pub deep_r_delay_us: u16,
    pub suspend_ret_r_delay_us: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct XtalStableTiming {
    pub delay_us: u32,
    pub loop_count: u32,
    pub nop_count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SleepRequest {
    pub mode: SleepMode,
    pub wakeup_src: WakeupSource,
    pub wakeup_tick: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SleepResult {
    pub raw: u32,
}

pub struct Pm;

impl Pm {
    #[inline(always)]
    pub fn init(source: Clock32kSource) {
        init(source);
    }

    #[inline(always)]
    pub fn configure_gpio_wakeup(raw_pin: gpio::RawPin, level: WakeupLevel, enabled: bool) {
        configure_gpio_wakeup(raw_pin, level, enabled);
    }

    #[inline(always)]
    pub fn sleep(request: SleepRequest) -> SleepResult {
        SleepResult {
            raw: sleep_until_tick(request.mode, request.wakeup_src, request.wakeup_tick),
        }
    }

    #[inline(always)]
    pub fn sleep_for_ms(mode: SleepMode, wakeup_src: WakeupSource, duration_ms: u32) -> SleepResult {
        SleepResult {
            raw: sleep_for_ms(mode, wakeup_src, duration_ms),
        }
    }

    #[inline(always)]
    pub fn long_sleep_32k(
        mode: SleepMode,
        wakeup_src: WakeupSource,
        duration_ticks_32k: u32,
    ) -> SleepResult {
        SleepResult {
            raw: long_sleep_32k(mode, wakeup_src, duration_ticks_32k),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn bls_pm_registerFuncBeforeSuspend(func: usize) {
    unsafe {
        core::ptr::write_volatile(&raw mut startup::func_before_suspend, func);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pm_set_wakeup_time_param(param: startup::PmRDelayUs) {
    unsafe {
        core::ptr::write_volatile(&raw mut startup::g_pm_r_delay_us, param);
        let deep = param.deep_r_delay_us;
        let suspend_ret = param.suspend_ret_r_delay_us;
        let suspend_delay = core::ptr::read_volatile(&raw const startup::g_pm_suspend_delay_us);
        startup::g_pm_early_wakeup_time_us.suspend =
            (u32::from(suspend_ret) + 0x00e6 + suspend_delay) as u16;
        startup::g_pm_early_wakeup_time_us.deep_ret = suspend_ret.wrapping_add(100);
        startup::g_pm_early_wakeup_time_us.deep = deep.wrapping_add(240);
        if startup::g_pm_early_wakeup_time_us.deep < startup::g_pm_early_wakeup_time_us.suspend {
            startup::g_pm_early_wakeup_time_us.min =
                startup::g_pm_early_wakeup_time_us.deep.wrapping_add(0x0190);
        } else {
            startup::g_pm_early_wakeup_time_us.min =
                startup::g_pm_early_wakeup_time_us.suspend.wrapping_add(0x0190);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pm_set_xtal_stable_timer_param(delay_us: u32, loopnum: u32, nopnum: u32) {
    unsafe {
        core::ptr::write_volatile(&raw mut startup::g_pm_xtal_stable_suspend_nopnum, nopnum);
        core::ptr::write_volatile(&raw mut startup::g_pm_xtal_stable_loopnum, loopnum);
        core::ptr::write_volatile(&raw mut startup::g_pm_suspend_delay_us, delay_us);
        startup::g_pm_early_wakeup_time_us.suspend =
            (startup::g_pm_r_delay_us.suspend_ret_r_delay_us as u32 + 0x00e6 + delay_us) as u16;
        if startup::g_pm_early_wakeup_time_us.deep < startup::g_pm_early_wakeup_time_us.suspend {
            startup::g_pm_early_wakeup_time_us.min =
                startup::g_pm_early_wakeup_time_us.deep.wrapping_add(0x0190);
        } else {
            startup::g_pm_early_wakeup_time_us.min =
                startup::g_pm_early_wakeup_time_us.suspend.wrapping_add(0x0190);
        }
    }
}

static mut CLOCK_32K_SOURCE: Clock32kSource = Clock32kSource::InternalRc;

#[inline(always)]
pub fn init(source: Clock32kSource) {
    #[cfg(feature = "chip-8258")]
    if source == Clock32kSource::InternalRc {
        // Vendor clock_32k_init(0): internal RC 32k.
        let clk32k_sel = analog::read(0x2d) & 0x7f;
        analog::write(0x2d, clk32k_sel);
        let mut pm32k_ctrl = analog::read(0x05) & !0x03;
        pm32k_ctrl |= 0x02;
        analog::write(0x05, pm32k_ctrl);
        rc_32k_cal_vendor_like();
    } else {
        // Vendor clock_32k_init(1): external 32k crystal + pad kick.
        let clk32k_sel = analog::read(0x2d) | 0x80;
        analog::write(0x2d, clk32k_sel);
        let mut pm32k_ctrl = analog::read(0x05) & !0x03;
        pm32k_ctrl |= 0x01;
        analog::write(0x05, pm32k_ctrl);
        ext_32k_kick_vendor_like();
    }
    select_32k_source(source);
}

#[cfg(feature = "chip-8258")]
fn rc_32k_cal_vendor_like() {
    analog::write(0x30, 0x60);
    analog::write(0xc6, 0xf6);
    analog::write(0xc6, 0xf7);
    while (analog::read(0xcf) & 0x40) == 0 {
        core::hint::spin_loop();
    }
    analog::write(0x32, analog::read(0xc9));
    analog::write(0x31, analog::read(0xca));
    analog::write(0xc6, 0xf6);
    analog::write(0x30, 0x20);
}

#[cfg(feature = "chip-8258")]
fn ext_32k_kick_vendor_like() {
    const REG_CLK_SEL: usize = 0x0080_0066;
    const REG_32K_PAD_CTRL: usize = 0x0080_0596;
    const REG_PWM_MAX_TICK: usize = 0x0080_0798;
    const REG_PWM_CMP_TICK: usize = 0x0080_079a;
    const REG_PWM_DATA: usize = 0x0080_0780;
    const REG_PWM_CTRL: usize = 0x0080_0782;

    let saved_clk;
    let saved_pad;
    let saved_max;
    let saved_cmp;
    let saved_pwm_data;
    unsafe {
        saved_clk = core::ptr::read_volatile(reg8(REG_CLK_SEL).cast_const());
        saved_pad = core::ptr::read_volatile(reg8(REG_32K_PAD_CTRL).cast_const());
        saved_max = core::ptr::read_volatile(reg16(REG_PWM_MAX_TICK).cast_const());
        saved_cmp = core::ptr::read_volatile(reg16(REG_PWM_CMP_TICK).cast_const());
        saved_pwm_data = core::ptr::read_volatile(reg8(REG_PWM_DATA).cast_const());

        core::ptr::write_volatile(reg8(REG_CLK_SEL), 0x43);
        core::ptr::write_volatile(reg8(REG_32K_PAD_CTRL), saved_pad & !0x08);
        core::ptr::write_volatile(reg16(REG_PWM_MAX_TICK), 0x0001);
        core::ptr::write_volatile(reg16(REG_PWM_CMP_TICK), 0x0002);
        core::ptr::write_volatile(reg8(REG_PWM_DATA), 0x02);
        core::ptr::write_volatile(reg8(REG_PWM_CTRL), 0xf3);
    }

    let started = timer::clock_time();
    while !timer::clock_time_exceed_us(started, 5_000) {
        core::hint::spin_loop();
    }
    analog::write(0x03, 0x4f);

    unsafe {
        core::ptr::write_volatile(reg8(REG_CLK_SEL), saved_clk);
        core::ptr::write_volatile(reg8(REG_32K_PAD_CTRL), saved_pad);
        core::ptr::write_volatile(reg16(REG_PWM_MAX_TICK), saved_max);
        core::ptr::write_volatile(reg16(REG_PWM_CMP_TICK), saved_cmp);
        core::ptr::write_volatile(reg8(REG_PWM_DATA), saved_pwm_data);
    }
}

#[inline(always)]
pub fn state() -> startup::StartupState {
    startup::startup_state()
}

#[inline(always)]
pub fn wake_origin() -> WakeOrigin {
    match startup::startup_state() {
        startup::StartupState::Boot => WakeOrigin::ColdBoot,
        startup::StartupState::Deep => WakeOrigin::DeepWake,
        startup::StartupState::DeepRetention => WakeOrigin::DeepRetentionWake,
    }
}

#[inline(always)]
pub fn is_cold_boot() -> bool {
    matches!(wake_origin(), WakeOrigin::ColdBoot)
}

#[inline(always)]
pub fn sync_sys_tick_per_us() {
    unsafe {
        startup::sysTimerPerUs = timer::sys_tick_per_us();
    }
}

#[inline(always)]
pub fn wakeup_source_raw() -> u8 {
    startup::wakeup_src_raw()
}

#[inline(always)]
pub fn is_pad_wakeup() -> bool {
    startup::is_pad_wakeup()
}

#[inline(always)]
pub fn current_32k_source() -> Clock32kSource {
    unsafe { core::ptr::read_volatile(&raw const CLOCK_32K_SOURCE) }
}

#[inline(always)]
pub fn set_wakeup_timing(timing: WakeupTiming) {
    pm_set_wakeup_time_param(startup::PmRDelayUs {
        deep_r_delay_us: timing.deep_r_delay_us,
        suspend_ret_r_delay_us: timing.suspend_ret_r_delay_us,
    });
}

#[inline(always)]
pub fn set_xtal_stable_timing(timing: XtalStableTiming) {
    pm_set_xtal_stable_timer_param(timing.delay_us, timing.loop_count, timing.nop_count);
}

#[inline(always)]
pub fn configure_gpio_wakeup(raw_pin: gpio::RawPin, level: WakeupLevel, enabled: bool) {
    cpu_set_gpio_wakeup(raw_pin.as_u16() as u32, level as u32, i32::from(enabled));
}

#[inline(always)]
pub fn sleep_until_tick(mode: SleepMode, wakeup_src: WakeupSource, wakeup_tick: u32) -> u32 {
    #[cfg(feature = "chip-8258")]
    {
        if mode.is_suspend() {
            startup::set_pm_long_suspend(false);
            return sleep_impl(
                mode,
                wakeup_src,
                wakeup_tick,
                current_32k_source(),
                false,
            ) as u32;
        }
        // Keep vendor short/deep path globals in sync. The ROM/object routine
        // uses cached tick globals early in its checks before internal updates.
        startup::set_tick_cur(timer::clock_time());
        startup::set_tick_32k_cur(current_32k_tick());
        startup::set_pm_long_suspend(false);
        match current_32k_source() {
            Clock32kSource::InternalRc => {
                cpu_sleep_wakeup_32k_rc(mode.raw() as u32, wakeup_src.raw() as u32, wakeup_tick)
                    as u32
            }
            Clock32kSource::ExternalCrystal => {
                cpu_sleep_wakeup_32k_xtal(mode.raw() as u32, wakeup_src.raw() as u32, wakeup_tick)
                    as u32
            }
        }
    }
    #[cfg(not(feature = "chip-8258"))]
    {
        let _ = (mode, wakeup_src, wakeup_tick);
        unimplemented!("pm::sleep_until_tick is only implemented for chip-8258")
    }
}

#[inline(always)]
pub fn sleep_for_ms(mode: SleepMode, wakeup_src: WakeupSource, duration_ms: u32) -> u32 {
    #[cfg(not(feature = "chip-8258"))]
    {
        let _ = (mode, wakeup_src, duration_ms);
        unimplemented!("pm::sleep_for_ms is only implemented for chip-8258")
    }
    #[cfg(feature = "chip-8258")]
    {
    if !mode.is_suspend() {
        let source = current_32k_source();
        if source == Clock32kSource::ExternalCrystal {
            // Vendor short XTAL path expects absolute system timer tick.
            let ticks_per_us = timer::sys_tick_per_us();
            let wakeup_tick = timer::clock_time().wrapping_add(
                duration_ms
                    .saturating_mul(1000)
                    .saturating_mul(ticks_per_us),
            );
            startup::set_tick_cur(timer::clock_time());
            startup::set_tick_32k_cur(current_32k_tick());
            startup::set_pm_long_suspend(false);
            return cpu_sleep_wakeup_32k_xtal(
                mode.raw() as u32,
                wakeup_src.raw() as u32,
                wakeup_tick,
            ) as u32;
        }
        let hz = hz_32k(source) as u64;
        let ticks_32k = ((duration_ms as u64).saturating_mul(hz) / 1000).max(1);
        return long_sleep_wakeup_impl(
            mode,
            wakeup_src,
            ticks_32k.min(u32::MAX as u64) as u32,
        ) as u32;
    }

    let ticks_per_us = timer::sys_tick_per_us();
    let wakeup_tick = timer::clock_time().wrapping_add(
        duration_ms
            .saturating_mul(1000)
            .saturating_mul(ticks_per_us),
    );
    sleep_until_tick(mode, wakeup_src, wakeup_tick)
    }
}

#[inline(always)]
pub fn long_sleep_32k(
    mode: SleepMode,
    wakeup_src: WakeupSource,
    duration_ticks_32k: u32,
) -> u32 {
    #[cfg(not(feature = "chip-8258"))]
    {
        let _ = (mode, wakeup_src, duration_ticks_32k);
        unimplemented!("pm::long_sleep_32k is only implemented for chip-8258")
    }
    #[cfg(feature = "chip-8258")]
    {
    long_sleep_wakeup_impl(mode, wakeup_src, duration_ticks_32k) as u32
    }
}

#[inline(always)]
pub fn pm_select_internal_32k_rc() {
    select_32k_source(Clock32kSource::InternalRc);
}

#[inline(always)]
pub fn pm_select_external_32k_crystal() {
    select_32k_source(Clock32kSource::ExternalCrystal);
}

#[inline(always)]
pub fn select_32k_source(source: Clock32kSource) {
    unsafe {
        core::ptr::write_volatile(&raw mut CLOCK_32K_SOURCE, source);
    }

    let tick_32k_calib = match source {
        Clock32kSource::InternalRc => 500u16,
        // 16MHz / 32768Hz ~= 488.281
        Clock32kSource::ExternalCrystal => 488u16,
    };
    startup::set_tick_32k_calib(tick_32k_calib);

    let recover = match source {
        Clock32kSource::InternalRc => pm_tim_recover_32k_rc as *const () as usize,
        Clock32kSource::ExternalCrystal => pm_tim_recover_32k_xtal as *const () as usize,
    };
    let sleep = match source {
        Clock32kSource::InternalRc => cpu_sleep_wakeup_32k_rc as *const () as usize,
        Clock32kSource::ExternalCrystal => cpu_sleep_wakeup_32k_xtal as *const () as usize,
    };
    let check_32k_handler = match source {
        Clock32kSource::InternalRc => 0usize,
        Clock32kSource::ExternalCrystal => check_32k_clk_stable as *const () as usize,
    };

    startup::set_pm_tim_recover_handler(recover);
    startup::set_cpu_sleep_wakeup_handler(sleep);
    startup::set_pm_check_32k_clk_stable_handler(check_32k_handler);
    startup::set_misc_pad32k_enabled(matches!(source, Clock32kSource::ExternalCrystal));
    startup::set_misc_pm_enter_enabled(true);
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
fn current_32k_tick() -> u32 {
    pm_get_32k_tick()
}

#[inline(always)]
fn hz_32k(source: Clock32kSource) -> u32 {
    match source {
        Clock32kSource::InternalRc => RC_32K_HZ,
        Clock32kSource::ExternalCrystal => XTAL_32K_HZ,
    }
}

#[inline(always)]
fn ticks_32k_to_sys_ticks(ticks_32k: u32, source: Clock32kSource) -> u32 {
    let hz = hz_32k(source) as u64;
    let sys_ticks = (ticks_32k as u64).saturating_mul(SYS_TICK_HZ as u64) / hz;
    sys_ticks.min(u32::MAX as u64) as u32
}

#[inline(always)]
#[cfg(test)]
fn sys_ticks_to_32k_ticks(sys_ticks: u32, source: Clock32kSource) -> u32 {
    let hz = hz_32k(source) as u64;
    let ticks_32k = (sys_ticks as u64).saturating_mul(hz) / (SYS_TICK_HZ as u64);
    ticks_32k.min(u32::MAX as u64) as u32
}

#[cfg(feature = "chip-8258")]
fn prepare_sleep(
    wakeup_src: WakeupSource,
    wakeup_tick: u32,
    source: Clock32kSource,
    long_sleep: bool,
) {
    startup::set_tick_cur(timer::clock_time());
    startup::set_tick_32k_cur(current_32k_tick());
    startup::set_pm_long_suspend(long_sleep);

    unsafe {
        core::ptr::write_volatile(reg32(REG_MCU_WAKEUP_MASK), wakeup_src.raw() as u32);
        if wakeup_src.contains(WakeupSource::TIMER) {
            let program_tick = if long_sleep {
                let delta_32k = wakeup_tick.wrapping_sub(startup::current_tick_32k_cur());
                startup::current_tick_cur().wrapping_add(ticks_32k_to_sys_ticks(delta_32k, source))
            } else {
                wakeup_tick
            };
            core::ptr::write_volatile(reg32(REG_SYSTEM_WAKEUP_TICK), program_tick);
        }
    }
}

#[cfg(feature = "chip-8258")]
fn enter_sleep(mode: SleepMode, wakeup_src: WakeupSource, wakeup_tick: u32) -> u32 {
    if mode.is_suspend() {
        let ticks_per_us = timer::sys_tick_per_us();
        let now = timer::clock_time();
        let delta_ticks = wakeup_tick.wrapping_sub(now);
        let interval_us = (delta_ticks / ticks_per_us).max(1);
        let stall_mask = if wakeup_src.contains(WakeupSource::TIMER) {
            // cpu_stall uses timer IRQ mask bits, not PM wake source bits.
            0x02
        } else {
            0
        };
        let wake = cpu_stall(stall_mask, interval_us, ticks_per_us);
        return wake | STATUS_ENTER_SUSPEND;
    }

    unsafe {
        core::ptr::write_volatile(reg8(REG_PWDN_CTRL), mode.raw());
    }

    loop {
        core::hint::spin_loop();
    }
}

#[cfg(feature = "chip-8258")]
fn sleep_impl(
    mode: SleepMode,
    wakeup_src: WakeupSource,
    wakeup_tick: u32,
    source: Clock32kSource,
    long_sleep: bool,
) -> i32 {
    if mode.is_suspend() {
        return enter_sleep(mode, wakeup_src, wakeup_tick) as i32;
    }
    prepare_sleep(wakeup_src, wakeup_tick, source, long_sleep);
    enter_sleep(mode, wakeup_src, wakeup_tick) as i32
}

#[cfg(not(feature = "chip-8258"))]
fn sleep_impl(
    _mode: SleepMode,
    _wakeup_src: WakeupSource,
    _wakeup_tick: u32,
    _source: Clock32kSource,
    _long_sleep: bool,
) -> i32 {
    unimplemented!("power management is only implemented for chip-8258 in this iteration");
}

#[unsafe(no_mangle)]
pub extern "C" fn pm_tim_recover_32k_rc(now_tick_32k: u32) -> u32 {
    unsafe {
        let deep_ret_tick = if startup::pm_long_suspend != 0 {
            startup::tick_cur.wrapping_add(
                now_tick_32k
                    .wrapping_sub(startup::tick_32k_cur)
                    .wrapping_div(16)
                    .wrapping_mul(startup::tick_32k_calib as u32),
            )
        } else {
            startup::tick_cur.wrapping_add(
                now_tick_32k
                    .wrapping_sub(startup::tick_32k_cur)
                    .wrapping_mul(startup::tick_32k_calib as u32)
                    .wrapping_div(16),
            )
        };
        startup::tick_cur = deep_ret_tick;
        startup::tick_32k_cur = now_tick_32k;
        deep_ret_tick
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pm_tim_recover_32k_xtal(now_tick_32k: u32) -> u32 {
    unsafe {
        let deep_ret_tick = if startup::pm_long_suspend != 0 {
            startup::tick_cur.wrapping_add(
                now_tick_32k
                    .wrapping_sub(startup::tick_32k_cur)
                    .wrapping_div(32)
                    .wrapping_mul(CRYSTAL32768_TICK_PER_32CYCLE),
            )
        } else {
            startup::tick_cur.wrapping_add(
                now_tick_32k
                    .wrapping_sub(startup::tick_32k_cur)
                    .wrapping_mul(CRYSTAL32768_TICK_PER_32CYCLE)
                    .wrapping_div(32),
            )
        };
        startup::tick_cur = deep_ret_tick;
        startup::tick_32k_cur = now_tick_32k;
        deep_ret_tick
    }
}

#[inline(always)]
fn long_sleep_wakeup_impl(
    mode: SleepMode,
    wakeup_src: WakeupSource,
    wakeup_duration_ticks_32k: u32,
) -> i32 {
    if !mode.is_suspend() {
        return match current_32k_source() {
            Clock32kSource::InternalRc => pm_long_sleep_wakeup(
                mode.raw() as u32,
                wakeup_src.raw() as u32,
                wakeup_duration_ticks_32k,
            ),
            Clock32kSource::ExternalCrystal => cpu_long_sleep_wakeup_32k_xtal(
                mode.raw() as u32,
                wakeup_src.raw() as u32,
                wakeup_duration_ticks_32k,
            ),
        };
    }
    let wakeup_tick_32k = current_32k_tick().wrapping_add(wakeup_duration_ticks_32k.max(1));
    let source = current_32k_source();
    sleep_impl(mode, wakeup_src, wakeup_tick_32k, source, true)
}

#[cfg(feature = "chip-8258")]
#[unsafe(no_mangle)]
pub extern "C" fn cpu_sleep_wakeup_32k_rc(mode: u32, wakeup_src: u32, wakeup_tick: u32) -> i32 {
    // VENDOR-DIFF: logic-equivalent Rust path; instruction layout may differ from GCC tc32.
    sleep_impl(
        decode_mode(mode),
        WakeupSource(wakeup_src as u8),
        wakeup_tick,
        Clock32kSource::InternalRc,
        false,
    )
}

#[cfg(feature = "chip-8258")]
#[unsafe(no_mangle)]
pub extern "C" fn cpu_sleep_wakeup_32k_xtal(mode: u32, wakeup_src: u32, wakeup_tick: u32) -> i32 {
    // VENDOR-DIFF: logic-equivalent Rust path; instruction layout may differ from GCC tc32.
    sleep_impl(
        decode_mode(mode),
        WakeupSource(wakeup_src as u8),
        wakeup_tick,
        Clock32kSource::ExternalCrystal,
        false,
    )
}

#[cfg(feature = "chip-8258")]
#[unsafe(no_mangle)]
pub extern "C" fn pm_long_sleep_wakeup(
    mode: u32,
    wakeup_src: u32,
    wakeup_duration_ticks_32k: u32,
) -> i32 {
    let sleep_mode = decode_mode(mode);
    let wakeup_src_u8 = wakeup_src as u8;
    let irq_enabled = interrupt::disable();
    let start_tick = unsafe { core::ptr::read_volatile(reg32(0x0080_0740).cast_const()) };
    unsafe {
        startup::tick_32k_calib = core::ptr::read_volatile(reg16(0x0080_0750).cast_const());
    }
    let has_timer = (wakeup_src_u8 & PM_WAKEUP_TIMER_BITS) != 0;
    if has_timer && wakeup_duration_ticks_32k < 0x40 {
        analog::write(0x44, WAKEUP_STATUS_ALL);
        let t = wakeup_duration_ticks_32k.wrapping_mul(31);
        let budget = t.wrapping_mul(4).wrapping_add(wakeup_duration_ticks_32k);
        let st = loop {
            let st = analog::read(0x44) & WAKEUP_STATUS_ALL;
            let now = unsafe { core::ptr::read_volatile(reg32(0x0080_0740).cast_const()) };
            if now.wrapping_sub(start_tick) >= budget || st != 0 {
                break st;
            }
        };
        interrupt::restore(irq_enabled);
        return st as i32;
    }
    unsafe {
        startup::pm_long_suspend = 0;
    }
    let before = unsafe { core::ptr::read_volatile(&raw const startup::func_before_suspend) };
    if before != 0 {
        let func: extern "C" fn() -> i32 = unsafe { core::mem::transmute(before) };
        if func() == 0 {
            interrupt::restore(irq_enabled);
            return WAKEUP_STATUS_PAD as i32;
        }
    }
    let now = unsafe { core::ptr::read_volatile(reg32(0x0080_0740).cast_const()) };
    unsafe {
        startup::tick_cur = now.wrapping_add(0x8c << 2);
        startup::tick_32k_cur = pm_get_32k_tick();
    }
    let wakeup_src_u32 = wakeup_src_u8 as u32;
    let minus64 = wakeup_duration_ticks_32k.wrapping_sub(0x40);
    analog::write(0x26, wakeup_src_u8);
    analog::write(0x44, WAKEUP_STATUS_ALL);
    let bak66 = unsafe { core::ptr::read_volatile(reg8(0x0080_0066).cast_const()) };
    unsafe {
        core::ptr::write_volatile(reg8(0x0080_0066), 0);
    }
    let sleep_mode_u8 = sleep_mode.raw();
    let sleep_mode_no_ret = sleep_mode_u8 & 0x7f;
    let mut an7 = 0u8;
    let mut v2c_base = 0x16u8;
    if sleep_mode_no_ret != 0 {
        let t2 = analog::read(0x02);
        analog::write(0x02, (t2 & !0x07) | 0x05);
        unsafe { core::ptr::write_volatile(reg8(0x0080_063e), startup::tl_multi_addr) };
        an7 = 5;
        v2c_base = 0x56;
        analog::write(0x2b, 0xde);
    } else {
        analog::write(0x04, 0x48);
        analog::write(0x7e, 0x00);
        an7 = 4;
        v2c_base = 0x96;
        analog::write(0x2b, 0x5e);
    }
    analog::write(0x7e, sleep_mode_u8);
    let cmp = ((wakeup_src_u32 & PM_WAKEUP_COMPARATOR_BITS as u32) != 0) as u8;
    let any = cmp | (has_timer as u8);
    analog::write(0x2c, v2c_base | any | (cmp << 3));
    analog::write(0x07, (analog::read(0x07) & !0x07) | an7);
    if sleep_mode_no_ret == 0 {
        unsafe {
            core::ptr::write_volatile(reg8(0x0080_0602), 0x08);
        }
        analog::write(0x7f, 0x01);
    } else {
        analog::write(0x7f, 0x00);
    }
    unsafe {
        let half = (startup::tick_32k_calib >> 1) as u32;
        if sleep_mode_u8 != 0 {
            analog::write(0x3c, analog::read(0x3c) | 0x02);
        }
        let v20 = 0x7f_u32.wrapping_sub((0xfa00_u32.wrapping_add(half)) / startup::tick_32k_calib as u32) as u8;
        analog::write(0x20, v20);
        let sr = startup::g_pm_r_delay_us.suspend_ret_r_delay_us as u32;
        analog::write(0x1f, (((sr << 7).wrapping_add(half)) / startup::tick_32k_calib as u32) as u8);
        let dt = core::ptr::read_volatile(reg32(0x0080_0740).cast_const()).wrapping_sub(start_tick);
        let wake_tick = if startup::pm_long_suspend != 0 {
            minus64.wrapping_add(startup::tick_cur).wrapping_sub((dt / startup::tick_32k_calib as u32) << 4)
        } else {
            minus64
                .wrapping_add(startup::tick_cur)
                .wrapping_sub(((dt << 4).wrapping_add((startup::tick_32k_calib >> 1) as u32) / startup::tick_32k_calib as u32))
        };
        core::ptr::write_volatile(reg8(0x0080_074c), 0x2c);
        core::ptr::write_volatile(reg32(0x0080_0754), wake_tick);
        core::ptr::write_volatile(reg8(0x0080_074f), 0x08);
        while (core::ptr::read_volatile(reg8(0x0080_074f).cast_const()) & 0x08) != 0 {}
        core::ptr::write_volatile(reg8(0x0080_074c), 0x20);
    }
    if (analog::read(0x44) & !WAKEUP_STATUS_ALL) == 0 {
        sleep_start();
    }
    if sleep_mode_u8 != 0 {
        analog::write(0x3c, analog::read(0x3c) & !0x02);
        soft_reboot_dly13ms_use24mRC();
        unsafe { core::ptr::write_volatile(reg8(REG_PWDN_CTRL), 0x20) };
    }
    unsafe {
        let t32 = pm_get_32k_tick();
        if startup::pm_long_suspend != 0 {
            startup::tick_cur = startup::tick_cur.wrapping_add(
                ((t32.wrapping_sub(startup::tick_32k_cur)) >> 4).wrapping_mul(startup::tick_32k_calib as u32),
            );
        } else {
            startup::tick_cur = startup::tick_cur.wrapping_add(
                (t32.wrapping_sub(startup::tick_32k_cur))
                    .wrapping_mul(startup::tick_32k_calib as u32)
                    >> 4,
            );
        }
        startup::tick_32k_cur = startup::tick_cur.wrapping_add(20 * 16);
        core::ptr::write_volatile(reg8(0x0080_074c), 0x00);
        core::ptr::write_volatile(reg8(0x0080_074c), 0x90);
        core::ptr::write_volatile(reg8(0x0080_074f), 0x01);
    }
    pm_wait_xtal_ready();
    unsafe {
        core::ptr::write_volatile(reg8(0x0080_0066), bak66);
    }
    let st = analog::read(0x44);
    interrupt::restore(irq_enabled);
    if st != 0 {
        (st as u32 | STATUS_ENTER_SUSPEND) as i32
    } else {
        STATUS_GPIO_ERR_NO_ENTER_PM as i32
    }
}

#[cfg(feature = "chip-8258")]
#[unsafe(no_mangle)]
pub extern "C" fn cpu_long_sleep_wakeup_32k_xtal(
    mode: u32,
    wakeup_src: u32,
    wakeup_duration_ticks_32k: u32,
) -> i32 {
    let sleep_mode = decode_mode(mode);
    let wakeup_src_u8 = wakeup_src as u8;
    let irq_enabled = interrupt::disable();
    let start = unsafe { core::ptr::read_volatile(reg32(0x0080_0740).cast_const()) };
    let has_timer = (wakeup_src_u8 & PM_WAKEUP_TIMER_BITS) != 0;
    if has_timer && wakeup_duration_ticks_32k < 0x40 {
        analog::write(0x44, WAKEUP_STATUS_ALL);
        let t = wakeup_duration_ticks_32k.wrapping_mul(31);
        let budget = (((t << 6).wrapping_sub(t)) << 3).wrapping_add(wakeup_duration_ticks_32k) >> 5;
        let st = loop {
            let st = analog::read(0x44) & WAKEUP_STATUS_ALL;
            let now = unsafe { core::ptr::read_volatile(reg32(0x0080_0740).cast_const()) };
            if now.wrapping_sub(start) >= budget || st != 0 {
                break st;
            }
        };
        interrupt::restore(irq_enabled);
        return st as i32;
    }
    unsafe {
        startup::pm_long_suspend = 0;
    }
    let before = unsafe { core::ptr::read_volatile(&raw const startup::func_before_suspend) };
    if before != 0 {
        let func: extern "C" fn() -> i32 = unsafe { core::mem::transmute(before) };
        if func() == 0 {
            interrupt::restore(irq_enabled);
            return WAKEUP_STATUS_PAD as i32;
        }
    }
    unsafe {
        let now = core::ptr::read_volatile(reg32(0x0080_0740).cast_const());
        startup::tick_cur = now.wrapping_add(0x8c << 2);
        startup::tick_32k_cur = pm_get_32k_tick();
    }
    let wake_m64 = wakeup_duration_ticks_32k.wrapping_sub(0x40);
    analog::write(0x26, wakeup_src_u8);
    analog::write(0x44, WAKEUP_STATUS_ALL);
    let bak66 = unsafe { core::ptr::read_volatile(reg8(0x0080_0066).cast_const()) };
    unsafe {
        core::ptr::write_volatile(reg8(0x0080_0066), 0);
    }
    let mut sleep_mode_u8 = sleep_mode.raw();
    let mut sleep_mode_no_ret = sleep_mode_u8 & 0x7f;
    let mut an7 = 0u8;
    let mut mode2c = 0x1d_u8;
    if sleep_mode_no_ret != 0 {
        let t2 = analog::read(0x02);
        analog::write(0x02, (t2 & !0x07) | 0x05);
        unsafe { core::ptr::write_volatile(reg8(0x0080_063e), startup::tl_multi_addr) };
        an7 = 5;
        mode2c = 0xde;
        let d = sleep_mode_u8.wrapping_sub(0x80);
        sleep_mode_u8 = (d | (0u8.wrapping_sub(d))) & 0xff;
    } else if sleep_mode_u8 == 0 {
        analog::write(0x04, 0x48);
        analog::write(0x7e, 0x00);
        analog::write(0x2b, 0x5e);
        an7 = 4;
        mode2c = 0x1d;
    } else {
        analog::write(0x7e, 0x80);
        analog::write(0x2b, 0xde);
        sleep_mode_no_ret = 1;
        an7 = 5;
        mode2c = 0xc0;
    }
    let wake_cmp = ((wakeup_src_u8 & PM_WAKEUP_COMPARATOR_BITS) != 0) as u8;
    if sleep_mode_u8 == 0 {
        analog::write(
            0x2c,
            0x80u8 | wake_cmp | PM_WAKEUP_TIMER_BITS | (wake_cmp << 3),
        );
    } else {
        analog::write(0x2c, 0x16u8 | mode2c);
    }
    analog::write(0x07, (analog::read(0x07) & !0x07) | an7);
    if sleep_mode_no_ret == 0 {
        unsafe { core::ptr::write_volatile(reg8(0x0080_0602), 0x08) };
        analog::write(0x7f, 1);
    } else {
        analog::write(0x7f, 0);
    }
    if sleep_mode_u8 != 0 {
        analog::write(0x3c, analog::read(0x3c) | 0x02);
    }
    analog::write(0x20, 0x77);
    unsafe {
        let sr = startup::g_pm_r_delay_us.suspend_ret_r_delay_us as u32;
        analog::write(0x1f, (((sr << 8) + (CRYSTAL32768_TICK_PER_32CYCLE >> 1)) / CRYSTAL32768_TICK_PER_32CYCLE) as u8);
        let dt = core::ptr::read_volatile(reg32(0x0080_0740).cast_const()).wrapping_sub(start);
        let wake_tick = if startup::pm_long_suspend != 0 {
            wake_m64.wrapping_add(startup::tick_cur).wrapping_sub((dt / CRYSTAL32768_TICK_PER_32CYCLE) << 5)
        } else {
            wake_m64
                .wrapping_add(startup::tick_cur)
                .wrapping_sub(((dt << 5).wrapping_add(CRYSTAL32768_TICK_PER_32CYCLE >> 1) / CRYSTAL32768_TICK_PER_32CYCLE))
        };
        core::ptr::write_volatile(reg8(0x0080_074c), 0x2c);
        core::ptr::write_volatile(reg32(0x0080_0754), wake_tick);
        core::ptr::write_volatile(reg8(0x0080_074f), 0x08);
        while (core::ptr::read_volatile(reg8(0x0080_074f).cast_const()) & 0x08) != 0 {}
        core::ptr::write_volatile(reg8(0x0080_074c), 0x20);
    }
    if (analog::read(0x44) & !WAKEUP_STATUS_ALL) == 0 {
        sleep_start();
    }
    if sleep_mode_u8 != 0 {
        analog::write(0x3c, analog::read(0x3c) & !0x03);
        soft_reboot_dly13ms_use24mRC();
        unsafe { core::ptr::write_volatile(reg8(REG_PWDN_CTRL), 0x20) };
    }
    unsafe {
        let t32 = pm_get_32k_tick();
        let d = t32.wrapping_sub(startup::tick_32k_cur);
        if startup::pm_long_suspend != 0 {
            startup::tick_cur = startup::tick_cur.wrapping_add(
                d.wrapping_div(32).wrapping_mul(CRYSTAL32768_TICK_PER_32CYCLE),
            );
        } else {
            startup::tick_cur = startup::tick_cur.wrapping_add(
                d.wrapping_mul(CRYSTAL32768_TICK_PER_32CYCLE).wrapping_div(32),
            );
        }
        startup::tick_32k_cur = startup::tick_cur.wrapping_add(20 * 16);
        core::ptr::write_volatile(reg8(0x0080_074c), 0x00);
        core::ptr::write_volatile(reg8(0x0080_074c), 0x90);
        core::ptr::write_volatile(reg8(0x0080_074f), 0x01);
    }
    pm_wait_xtal_ready();
    unsafe {
        core::ptr::write_volatile(reg8(0x0080_0066), bak66);
    }
    let st = analog::read(0x44);
    interrupt::restore(irq_enabled);
    if st != 0 {
        (st as u32 | STATUS_ENTER_SUSPEND) as i32
    } else {
        STATUS_GPIO_ERR_NO_ENTER_PM as i32
    }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
fn decode_mode(raw: u32) -> SleepMode {
    match raw as u8 {
        0x00 => SleepMode::Suspend,
        0x80 => SleepMode::DeepSleep,
        0x61 => SleepMode::DeepSleepRetentionLow8K,
        0x43 => SleepMode::DeepSleepRetentionLow16K,
        0x07 => SleepMode::DeepSleepRetentionLow32K,
        0xff => SleepMode::Shutdown,
        // VENDOR-DIFF: invalid value is normalized to suspend.
        _ => SleepMode::Suspend,
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code.check_32k_clk_stable")]
pub extern "C" fn check_32k_clk_stable() {
    startup::startup_pm_wait_xtal_ready();
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code.pm_get_32k_tick")]
pub extern "C" fn pm_get_32k_tick() -> u32 {
    startup::startup_pm_get_32k_tick()
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code.pm_wait_xtal_ready")]
pub extern "C" fn pm_wait_xtal_ready() {
    startup::startup_pm_wait_xtal_ready();
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code.start_reboot")]
pub extern "C" fn start_reboot() -> ! {
    startup::startup_start_reboot()
}

#[unsafe(no_mangle)]
pub extern "C" fn soft_reboot_dly13ms_use24mRC() {
    startup::startup_soft_reboot_dly13ms_use24m_rc();
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code.sleep_start")]
pub extern "C" fn sleep_start() {
    startup::startup_sleep_start();
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code.cpu_stall")]
pub extern "C" fn cpu_stall(wakeup_src: u32, interval_us: u32, sysclktick: u32) -> u32 {
    startup::startup_cpu_stall(wakeup_src, interval_us, sysclktick)
}

#[unsafe(no_mangle)]
pub extern "C" fn cpu_stall_wakeup_by_timer0(tick: u32) {
    startup::startup_cpu_stall_wakeup_by_timer0(tick);
}

#[unsafe(no_mangle)]
pub extern "C" fn cpu_stall_wakeup_by_timer1(tick: u32) {
    startup::startup_cpu_stall_wakeup_by_timer1(tick);
}

#[unsafe(no_mangle)]
pub extern "C" fn cpu_stall_wakeup_by_timer2(tick: u32) {
    startup::startup_cpu_stall_wakeup_by_timer2(tick);
}

#[unsafe(no_mangle)]
pub extern "C" fn cpu_set_gpio_wakeup(pin: u32, pol: u32, en: i32) {
    startup::startup_cpu_set_gpio_wakeup(pin, pol, en);
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code.cpu_wakeup_init")]
pub extern "C" fn cpu_wakeup_init() {
    startup::startup_cpu_wakeup_init();
}

#[cfg(test)]
mod tests {
    use super::{sys_ticks_to_32k_ticks, ticks_32k_to_sys_ticks, Clock32kSource};

    #[test]
    fn rc_32k_tick_conversion_matches_16mhz_ratio() {
        assert_eq!(
            ticks_32k_to_sys_ticks(32, Clock32kSource::InternalRc),
            16_000
        );
        assert_eq!(
            sys_ticks_to_32k_ticks(16_000, Clock32kSource::InternalRc),
            32
        );
    }

    #[test]
    fn xtal_32k_tick_conversion_matches_expected_rounding_window() {
        assert_eq!(
            ticks_32k_to_sys_ticks(32_768, Clock32kSource::ExternalCrystal),
            16_000_000
        );
        assert_eq!(
            sys_ticks_to_32k_ticks(16_000_000, Clock32kSource::ExternalCrystal),
            32_768
        );
    }
}
