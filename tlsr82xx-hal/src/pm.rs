use crate::{analog, gpio, interrupt, startup, timer};

#[cfg(feature = "chip-8258")]
use crate::mmio::{reg16, reg32, reg8};
#[cfg(feature = "chip-8258")]
use crate::regs8258::{REG_MCU_WAKEUP_MASK, REG_PWDN_CTRL};

#[cfg(feature = "chip-8258")]
const REG_SYSTEM_WAKEUP_TICK: usize = 0x0080_0748;

const RC_32K_HZ: u32 = 32_000;
const XTAL_32K_HZ: u32 = 32_768;
const SYS_TICK_HZ: u32 = 16_000_000;
const PM_NORMAL_SLEEP_MAX_MS: u32 = 230 * 1000;

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

    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
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
pub struct Config {
    pub clock_32k: Clock32kSource,
    pub wakeup_timing: WakeupTiming,
    pub xtal_stable_timing: XtalStableTiming,
}

impl Config {
    #[inline(always)]
    pub const fn internal_rc() -> Self {
        Self {
            clock_32k: Clock32kSource::InternalRc,
            wakeup_timing: WakeupTiming {
                deep_r_delay_us: 1000,
                suspend_ret_r_delay_us: 1000,
            },
            xtal_stable_timing: XtalStableTiming {
                delay_us: 0x87,
                loop_count: 10,
                nop_count: 200,
            },
        }
    }

    #[inline(always)]
    pub const fn external_crystal() -> Self {
        Self {
            clock_32k: Clock32kSource::ExternalCrystal,
            ..Self::internal_rc()
        }
    }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SleepMsKind {
    Short,
    Long,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct WakeupStatus(u8);

impl WakeupStatus {
    #[inline(always)]
    pub const fn raw(self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub const fn contains_timer(self) -> bool {
        (self.0 & WAKEUP_STATUS_TIMER as u8) != 0
    }

    #[inline(always)]
    pub const fn contains_pad(self) -> bool {
        (self.0 & WAKEUP_STATUS_PAD as u8) != 0
    }

    #[inline(always)]
    pub const fn contains_core(self) -> bool {
        (self.0 & WAKEUP_STATUS_CORE as u8) != 0
    }

    #[inline(always)]
    pub const fn contains_comparator(self) -> bool {
        (self.0 & WAKEUP_STATUS_COMPARATOR as u8) != 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WakeInfo {
    pub origin: WakeOrigin,
    pub source: WakeupStatus,
    pub is_pad_wakeup: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Pm {
    config: Config,
}

impl Pm {
    #[inline(always)]
    pub fn init(config: Config) -> Self {
        apply_config(config);
        Self { config }
    }

    #[inline(always)]
    pub fn reconfigure(&mut self, config: Config) {
        apply_config(config);
        self.config = config;
    }

    #[inline(always)]
    pub fn config(&self) -> Config {
        self.config
    }

    #[inline(always)]
    pub fn wake_info(&self) -> WakeInfo {
        wake_info()
    }

    #[inline(always)]
    pub fn configure_gpio_wakeup(
        &mut self,
        raw_pin: gpio::RawPin,
        level: WakeupLevel,
        enabled: bool,
    ) {
        configure_gpio_wakeup(raw_pin, level, enabled);
    }

    #[inline(always)]
    pub fn sleep(&mut self, request: SleepRequest) -> SleepResult {
        SleepResult {
            raw: sleep_until_tick_impl(request.mode, request.wakeup_src, request.wakeup_tick),
        }
    }

    #[inline(always)]
    pub fn sleep_until_wakeup(&mut self, mode: SleepMode, wakeup_src: WakeupSource) -> SleepResult {
        SleepResult {
            raw: sleep_until_wakeup_impl(mode, wakeup_src),
        }
    }

    #[inline(always)]
    pub fn sleep_ms_short(
        &mut self,
        mode: SleepMode,
        wakeup_src: WakeupSource,
        duration_ms: u32,
    ) -> SleepResult {
        SleepResult {
            raw: sleep_for_ms_impl(mode, wakeup_src, duration_ms),
        }
    }

    #[inline(always)]
    pub fn sleep_ms(
        &mut self,
        mode: SleepMode,
        wakeup_src: WakeupSource,
        duration_ms: u32,
    ) -> SleepResult {
        match sleep_ms_kind(duration_ms) {
            SleepMsKind::Short => self.sleep_ms_short(mode, wakeup_src, duration_ms),
            SleepMsKind::Long => self.long_sleep_32k(
                mode,
                wakeup_src,
                duration_ms_to_32k_ticks(duration_ms, current_32k_source()),
            ),
        }
    }

    #[inline(always)]
    pub fn sleep_until_tick(
        &mut self,
        mode: SleepMode,
        wakeup_src: WakeupSource,
        wakeup_tick: u32,
    ) -> SleepResult {
        SleepResult {
            raw: sleep_until_tick_impl(mode, wakeup_src, wakeup_tick),
        }
    }

    #[inline(always)]
    pub fn long_sleep_32k(
        &mut self,
        mode: SleepMode,
        wakeup_src: WakeupSource,
        duration_ticks_32k: u32,
    ) -> SleepResult {
        SleepResult {
            raw: long_sleep_32k_impl(mode, wakeup_src, duration_ticks_32k),
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
            startup::g_pm_early_wakeup_time_us.min = startup::g_pm_early_wakeup_time_us
                .suspend
                .wrapping_add(0x0190);
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
            startup::g_pm_early_wakeup_time_us.min = startup::g_pm_early_wakeup_time_us
                .suspend
                .wrapping_add(0x0190);
        }
    }
}

static mut CLOCK_32K_SOURCE: Clock32kSource = Clock32kSource::InternalRc;

#[inline(always)]
fn apply_config(config: Config) {
    init_32k_source(config.clock_32k);
    set_wakeup_timing(config.wakeup_timing);
    set_xtal_stable_timing(config.xtal_stable_timing);
    sync_sys_tick_per_us();
}

#[inline(always)]
fn init_32k_source(source: Clock32kSource) {
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
pub fn wake_info() -> WakeInfo {
    WakeInfo {
        origin: wake_origin(),
        source: WakeupStatus(wakeup_source_raw()),
        is_pad_wakeup: is_pad_wakeup(),
    }
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
fn sleep_until_tick_impl(mode: SleepMode, wakeup_src: WakeupSource, wakeup_tick: u32) -> u32 {
    #[cfg(feature = "chip-8258")]
    {
        if mode.is_suspend() {
            startup::set_pm_long_suspend(false);
            return sleep_impl(mode, wakeup_src, wakeup_tick, current_32k_source(), false) as u32;
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
fn sleep_until_wakeup_impl(mode: SleepMode, wakeup_src: WakeupSource) -> u32 {
    #[cfg(not(feature = "chip-8258"))]
    {
        let _ = (mode, wakeup_src);
        unimplemented!("pm::sleep_until_wakeup is only implemented for chip-8258")
    }
    #[cfg(feature = "chip-8258")]
    {
        let wakeup_src = wakeup_src.without(WakeupSource::TIMER);
        if wakeup_src.is_empty() || mode.is_suspend() {
            return STATUS_GPIO_ERR_NO_ENTER_PM;
        }
        sleep_until_tick_impl(mode, wakeup_src, timer::clock_time())
    }
}

#[inline(always)]
fn sleep_for_ms_impl(mode: SleepMode, wakeup_src: WakeupSource, duration_ms: u32) -> u32 {
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
            return long_sleep_32k_impl(
                mode,
                wakeup_src,
                duration_ms_to_32k_ticks(duration_ms, source),
            );
        }

        let ticks_per_us = timer::sys_tick_per_us();
        let wakeup_tick = timer::clock_time().wrapping_add(
            duration_ms
                .saturating_mul(1000)
                .saturating_mul(ticks_per_us),
        );
        sleep_until_tick_impl(mode, wakeup_src, wakeup_tick)
    }
}

#[inline(always)]
fn long_sleep_32k_impl(mode: SleepMode, wakeup_src: WakeupSource, duration_ticks_32k: u32) -> u32 {
    #[cfg(not(feature = "chip-8258"))]
    {
        let _ = (mode, wakeup_src, duration_ticks_32k);
        unimplemented!("pm::long_sleep_32k is only implemented for chip-8258")
    }
    #[cfg(feature = "chip-8258")]
    {
        // Vendor 8258 long-sleep entry points consume raw 32k-domain durations
        // (`duration_ms * 32` in the official SDK RC path), so keep this API in
        // the same unit and pass it through unchanged.
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
        // Vendor RC PM code uses `tick_32k_calib` in the scaled
        // `delta_32k * calib / 16` form, so the seed must be the
        // system-tick count for 16 cycles of 32k, not the raw 16MHz/32k ratio.
        Clock32kSource::InternalRc => 8_000u16,
        // Vendor XTAL PM code uses the 32-cycle constant `15625`
        // (16MHz / 32768Hz * 32). Keep the same unit here so any
        // fallback reads stay on the vendor scale.
        Clock32kSource::ExternalCrystal => 15_625u16,
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
fn sleep_ms_kind(duration_ms: u32) -> SleepMsKind {
    if duration_ms > PM_NORMAL_SLEEP_MAX_MS {
        SleepMsKind::Long
    } else {
        SleepMsKind::Short
    }
}

#[inline(always)]
fn duration_ms_to_32k_ticks(duration_ms: u32, source: Clock32kSource) -> u32 {
    let hz = hz_32k(source) as u64;
    ((duration_ms as u64).saturating_mul(hz) / 1000)
        .max(1)
        .min(u32::MAX as u64) as u32
}

#[inline(always)]
fn ticks_32k_to_sys_ticks(ticks_32k: u32, source: Clock32kSource) -> u32 {
    let hz = hz_32k(source) as u64;
    // TLSR8258 datasheet, "5.3 System Timer":
    // system timer clock is fixed at 16MHz irrespective of system clock.
    let sys_hz = SYS_TICK_HZ as u64;
    let sys_ticks = (ticks_32k as u64).saturating_mul(sys_hz) / hz;
    sys_ticks.min(u32::MAX as u64) as u32
}

#[inline(always)]
fn vendor_clock_dly(cycles: usize) {
    let mut delay = 0usize;
    while unsafe { core::ptr::read_volatile(&raw const delay) } < cycles {
        let next = unsafe { core::ptr::read_volatile(&raw const delay) }.wrapping_add(1);
        unsafe { core::ptr::write_volatile(&raw mut delay, next) };
    }
}

#[inline(always)]
fn wake44_gate_ready(wake44: u8) -> bool {
    // Vendor checks only the documented wake-status bits in ana 0x44[3:0].
    // Bit7 has been observed set on real hardware and is irrelevant here.
    (wake44 & WAKEUP_STATUS_ALL) == 0
}

#[inline(always)]
#[cfg(test)]
fn sys_ticks_to_32k_ticks(sys_ticks: u32, source: Clock32kSource) -> u32 {
    let hz = hz_32k(source) as u64;
    let sys_hz = SYS_TICK_HZ as u64;
    let ticks_32k = (sys_ticks as u64).saturating_mul(hz) / sys_hz;
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
#[unsafe(link_section = ".text.pm_tim_recover_32k_rc")]
pub extern "C" fn pm_tim_recover_32k_rc(now_tick_32k: u32) -> u32 {
    unsafe {
        if startup::pm_long_suspend != 0 {
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
        }
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.pm_tim_recover_32k_xtal")]
pub extern "C" fn pm_tim_recover_32k_xtal(now_tick_32k: u32) -> u32 {
    unsafe {
        if startup::pm_long_suspend != 0 {
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
        }
    }
}

#[inline(always)]
fn long_sleep_wakeup_impl(
    mode: SleepMode,
    wakeup_src: WakeupSource,
    wakeup_duration_ticks_32k: u32,
) -> i32 {
    let source = current_32k_source();
    let duration_ticks_32k = wakeup_duration_ticks_32k.max(1);
    // Vendor long-sleep headers document raw 32k-tick durations for both RC
    // and XTAL variants (`32*1000 -> 1s` for RC, `32768 -> 1s` for XTAL).
    // Keep the long-sleep API in that domain.
    match source {
        Clock32kSource::InternalRc => pm_long_sleep_wakeup(
            mode.raw() as u32,
            wakeup_src.raw() as u32,
            duration_ticks_32k,
        ),
        Clock32kSource::ExternalCrystal => cpu_long_sleep_wakeup_32k_xtal(
            mode.raw() as u32,
            wakeup_src.raw() as u32,
            duration_ticks_32k,
        ),
    }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
#[unsafe(link_section = ".text.switch_ext32kpad_to_int32krc")]
fn switch_ext32kpad_to_int32krc(mode: u8) {
    // Vendor parity: ANA_SYS_DEEP_CLR(SYS_NEED_REINIT_EXT32K)
    analog::write(0x3c, analog::read(0x3c) & !0x01);
    analog::write(0x2d, 0x15);
    analog::write(0x05, 0x02);
    analog::write(0x2c, mode | 0x16);
}

#[cfg(feature = "chip-8258")]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.cpu_sleep_wakeup_32k_rc")]
pub extern "C" fn cpu_sleep_wakeup_32k_rc(mode: u32, wakeup_src: u32, wakeup_tick: u32) -> i32 {
    let sleep_mode_u8 = decode_mode(mode).raw();
    let wakeup_src_u8 = wakeup_src as u8;
    let irq_enabled = interrupt::disable();
    let wake_ticks = wakeup_tick;
    let timer_wakeup = (wakeup_src_u8 & PM_WAKEUP_TIMER_BITS) != 0;

    while unsafe { core::ptr::read_volatile(&raw const startup::tick_32k_calib) } == 0 {
        core::hint::spin_loop();
    }
    let calib = unsafe { core::ptr::read_volatile(&raw const startup::tick_32k_calib) };
    unsafe { core::ptr::write_volatile(&raw mut startup::tick_32k_calib, calib) };
    let t0 = unsafe { core::ptr::read_volatile(reg32(0x0080_0740).cast_const()) };

    if timer_wakeup {
        let dt = wake_ticks.wrapping_sub(t0);
        if dt > 0xE0000000 {
            interrupt::restore(irq_enabled);
            return (analog::read(0x44) & WAKEUP_STATUS_ALL) as i32;
        }
        let min_wakeup_us =
            unsafe { core::ptr::read_volatile(&raw const startup::g_pm_early_wakeup_time_us.min) };
        let ew = (min_wakeup_us as u32) << 4;
        if dt >= ew {
            unsafe {
                core::ptr::write_volatile(
                    &raw mut startup::pm_long_suspend,
                    u8::from(dt > (0xffu32 << 20)),
                );
            }
        } else {
            analog::write(0x44, WAKEUP_STATUS_ALL);
            let st = loop {
                let st = analog::read(0x44) & WAKEUP_STATUS_ALL;
                let now = unsafe { core::ptr::read_volatile(reg32(0x0080_0740).cast_const()) };
                if now.wrapping_sub(t0) >= dt || st != 0 {
                    break st;
                }
            };
            interrupt::restore(irq_enabled);
            return st as i32;
        }
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
        core::ptr::write_volatile(
            &raw mut startup::tick_cur,
            core::ptr::read_volatile(reg32(0x0080_0740).cast_const()).wrapping_add(0x8c << 2),
        );
        core::ptr::write_volatile(&raw mut startup::tick_32k_cur, pm_get_32k_tick());
    }

    let suspend_early =
        unsafe { core::ptr::read_volatile(&raw const startup::g_pm_early_wakeup_time_us.suspend) };
    let deep_ret_early =
        unsafe { core::ptr::read_volatile(&raw const startup::g_pm_early_wakeup_time_us.deep_ret) };
    let early = if sleep_mode_u8 != 0 {
        deep_ret_early
    } else {
        suspend_early
    } as u32;
    let target = wake_ticks.wrapping_sub(early << 4);

    analog::write(0x26, wakeup_src_u8);
    analog::write(0x44, WAKEUP_STATUS_ALL);
    let bak66 = unsafe { core::ptr::read_volatile(reg8(0x0080_0066).cast_const()) };
    unsafe { core::ptr::write_volatile(reg8(0x0080_0066), 0) };

    let sleep_mode_no_ret = sleep_mode_u8 & 0x7f;
    let (an7, a2c_high, a2b, a7e) = if sleep_mode_no_ret != 0 {
        let t2 = analog::read(0x02);
        analog::write(0x02, (t2 & !0x07) | 0x05);
        unsafe { core::ptr::write_volatile(reg8(0x0080_063e), startup::tl_multi_addr) };
        (5u8, 0x40u8, 0xdeu8, sleep_mode_u8)
    } else if sleep_mode_u8 == 0 {
        analog::write(0x04, 0x48);
        analog::write(0x7e, 0x00);
        (4u8, 0x96u8, 0x5eu8, 0u8)
    } else {
        (5u8, 0xc0u8, 0xdeu8, sleep_mode_u8)
    };
    analog::write(0x7e, a7e);
    analog::write(0x2b, a2b);
    let cmp = u8::from((wakeup_src_u8 & PM_WAKEUP_COMPARATOR_BITS) != 0);
    let any = cmp | u8::from(timer_wakeup);
    analog::write(0x2c, 0x16 | a2c_high | any | (cmp << 3));
    analog::write(0x07, (analog::read(0x07) & !0x07) | an7);
    if sleep_mode_no_ret == 0 {
        unsafe { core::ptr::write_volatile(reg8(0x0080_0602), 0x08) };
        analog::write(0x7f, 1);
    } else {
        analog::write(0x7f, 0);
    }

    let half = (calib >> 1) as u32;
    if sleep_mode_u8 != 0 {
        analog::write(0x3c, analog::read(0x3c) | 0x02);
    }
    analog::write(
        0x20,
        0x7f_u32.wrapping_sub((0xfa00u32.wrapping_add(half)) / (calib as u32)) as u8,
    );
    let sr = unsafe {
        core::ptr::read_volatile(&raw const startup::g_pm_r_delay_us.suspend_ret_r_delay_us)
    } as u32;
    analog::write(
        0x1f,
        (((sr << 7).wrapping_add(half)) / (calib as u32)) as u8,
    );

    let d = target.wrapping_sub(unsafe { core::ptr::read_volatile(&raw const startup::tick_cur) });
    let wake_tick = if unsafe { core::ptr::read_volatile(&raw const startup::pm_long_suspend) } != 0
    {
        target
            .wrapping_sub((d / (calib as u32)) << 4)
            .wrapping_add(unsafe { core::ptr::read_volatile(&raw const startup::tick_32k_cur) })
    } else {
        target
            .wrapping_sub(((d << 4).wrapping_add((calib >> 1) as u32)) / (calib as u32))
            .wrapping_add(unsafe { core::ptr::read_volatile(&raw const startup::tick_32k_cur) })
    };
    unsafe {
        core::ptr::write_volatile(reg8(0x0080_074c), 0x2c);
        core::ptr::write_volatile(reg32(0x0080_0754), wake_tick);
        core::ptr::write_volatile(reg8(0x0080_074f), 0x08);
        vendor_clock_dly(10);
        vendor_clock_dly(6);
        while (core::ptr::read_volatile(reg8(0x0080_074f).cast_const()) & 0x08) != 0 {}
        core::ptr::write_volatile(reg8(0x0080_074c), 0x20);
    }

    // Keep the gate-to-sleep sequence vendor-tight: extra volatile stores here
    // were enough to perturb PM entry on real hardware.
    if wake44_gate_ready(analog::read(0x44)) {
        sleep_start();
    }
    if sleep_mode_u8 != 0 {
        analog::write(0x3c, analog::read(0x3c) & !0x02);
        soft_reboot_dly13ms_use24mRC();
        unsafe { core::ptr::write_volatile(reg8(REG_PWDN_CTRL), 0x20) };
    }

    let t32 = pm_get_32k_tick();
    unsafe {
        let cur = core::ptr::read_volatile(&raw const startup::tick_cur);
        let cur32 = core::ptr::read_volatile(&raw const startup::tick_32k_cur);
        let long = core::ptr::read_volatile(&raw const startup::pm_long_suspend) != 0;
        let upd = if long {
            cur.wrapping_add(((t32.wrapping_sub(cur32)) >> 4).wrapping_mul(calib as u32))
        } else {
            cur.wrapping_add((t32.wrapping_sub(cur32)).wrapping_mul(calib as u32) >> 4)
        };
        core::ptr::write_volatile(&raw mut startup::tick_cur, upd);
        core::ptr::write_volatile(&raw mut startup::tick_32k_cur, upd.wrapping_add(20 * 16));
        core::ptr::write_volatile(reg8(0x0080_074c), 0x00);
        vendor_clock_dly(6);
        core::ptr::write_volatile(reg8(0x0080_074c), 0x92);
        vendor_clock_dly(4);
        core::ptr::write_volatile(reg8(0x0080_074f), 0x01);
    }
    pm_wait_xtal_ready();
    unsafe { core::ptr::write_volatile(reg8(0x0080_0066), bak66) };

    let st = analog::read(0x44);
    if (st & PM_WAKEUP_COMPARATOR_BITS) != 0 && timer_wakeup {
        loop {
            let now = unsafe { core::ptr::read_volatile(reg32(0x0080_0740).cast_const()) };
            if now.wrapping_sub(wake_ticks) <= (1u32 << 30) {
                break;
            }
        }
    }
    interrupt::restore(irq_enabled);
    if st != 0 {
        (st as u32 | STATUS_ENTER_SUSPEND) as i32
    } else {
        STATUS_GPIO_ERR_NO_ENTER_PM as i32
    }
}

#[cfg(feature = "chip-8258")]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.cpu_sleep_wakeup_32k_xtal")]
pub extern "C" fn cpu_sleep_wakeup_32k_xtal(mode: u32, wakeup_src: u32, wakeup_tick: u32) -> i32 {
    let sleep_mode_u8 = decode_mode(mode).raw();
    let wakeup_src_u8 = wakeup_src as u8;
    let irq_enabled = interrupt::disable();
    let timer_wakeup = (wakeup_src_u8 & PM_WAKEUP_TIMER_BITS) != 0;
    let start = unsafe { core::ptr::read_volatile(reg32(0x0080_0740).cast_const()) };

    if timer_wakeup {
        let dt = wakeup_tick.wrapping_sub(start);
        if dt > 0xE0000000 {
            interrupt::restore(irq_enabled);
            return (analog::read(0x44) & WAKEUP_STATUS_ALL) as i32;
        }
        let min_wakeup_us =
            unsafe { core::ptr::read_volatile(&raw const startup::g_pm_early_wakeup_time_us.min) };
        if dt >= ((min_wakeup_us as u32) << 4) {
            unsafe {
                core::ptr::write_volatile(
                    &raw mut startup::pm_long_suspend,
                    if dt > 0x07feffff {
                        1
                    } else {
                        u8::from(timer_wakeup)
                    },
                );
            }
        } else {
            analog::write(0x44, WAKEUP_STATUS_ALL);
            let st = loop {
                let st = analog::read(0x44) & WAKEUP_STATUS_ALL;
                let now = unsafe { core::ptr::read_volatile(reg32(0x0080_0740).cast_const()) };
                if now.wrapping_sub(start) >= dt || st != 0 {
                    break st;
                }
            };
            interrupt::restore(irq_enabled);
            return st as i32;
        }
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
        core::ptr::write_volatile(
            &raw mut startup::tick_cur,
            core::ptr::read_volatile(reg32(0x0080_0740).cast_const()).wrapping_add(0x8c << 2),
        );
        core::ptr::write_volatile(&raw mut startup::tick_32k_cur, pm_get_32k_tick());
    }
    let suspend_early =
        unsafe { core::ptr::read_volatile(&raw const startup::g_pm_early_wakeup_time_us.suspend) };
    let deep_early =
        unsafe { core::ptr::read_volatile(&raw const startup::g_pm_early_wakeup_time_us.deep) };
    let target = if sleep_mode_u8 == 0x80 {
        wakeup_tick.wrapping_sub((deep_early as u32) << 4)
    } else {
        wakeup_tick.wrapping_sub((suspend_early as u32) << 4)
    };

    analog::write(0x26, wakeup_src_u8);
    analog::write(0x44, WAKEUP_STATUS_ALL);
    let bak66 = unsafe { core::ptr::read_volatile(reg8(0x0080_0066).cast_const()) };
    unsafe { core::ptr::write_volatile(reg8(0x0080_0066), 0) };

    let sleep_mode_no_ret = sleep_mode_u8 & 0x7f;
    let an7 = if sleep_mode_no_ret != 0 {
        let t2 = analog::read(0x02);
        analog::write(0x02, (t2 & !0x07) | 0x05);
        unsafe { core::ptr::write_volatile(reg8(0x0080_063e), startup::tl_multi_addr) };
        analog::write(0x7e, sleep_mode_u8);
        analog::write(0x2b, 0xde);
        let delta = sleep_mode_u8.wrapping_sub(0x80);
        let not_deep = u8::from(delta != 0);
        analog::write(0x2c, 0xD6u8 | not_deep | (not_deep << 3));
        5u8
    } else {
        analog::write(0x04, 0x48);
        analog::write(0x7e, 0x00);
        analog::write(0x2b, 0x5e);
        analog::write(0x2c, 0x80u8 | if timer_wakeup { 0x14 } else { 0x1d });
        4u8
    };
    analog::write(0x07, (analog::read(0x07) & !0x07) | an7);
    if sleep_mode_no_ret == 0 {
        unsafe { core::ptr::write_volatile(reg8(0x0080_0602), 0x08) };
        analog::write(0x7f, 1);
    } else {
        analog::write(0x7f, 0);
    }

    if sleep_mode_u8 == 0x80 {
        analog::write(0x3c, analog::read(0x3c) | 0x02);
    }
    analog::write(0x20, 0x77);
    let sr = unsafe {
        core::ptr::read_volatile(&raw const startup::g_pm_r_delay_us.suspend_ret_r_delay_us)
    } as u32;
    analog::write(
        0x1f,
        (((sr << 8).wrapping_add(CRYSTAL32768_TICK_PER_32CYCLE >> 1))
            / CRYSTAL32768_TICK_PER_32CYCLE) as u8,
    );

    let d = target.wrapping_sub(unsafe { core::ptr::read_volatile(&raw const startup::tick_cur) });
    let wake_tick = if unsafe { core::ptr::read_volatile(&raw const startup::pm_long_suspend) } != 0
    {
        target
            .wrapping_sub((d / CRYSTAL32768_TICK_PER_32CYCLE) << 5)
            .wrapping_add(unsafe { core::ptr::read_volatile(&raw const startup::tick_32k_cur) })
    } else {
        target
            .wrapping_sub(
                ((d << 5).wrapping_add(CRYSTAL32768_TICK_PER_32CYCLE >> 1))
                    / CRYSTAL32768_TICK_PER_32CYCLE,
            )
            .wrapping_add(unsafe { core::ptr::read_volatile(&raw const startup::tick_32k_cur) })
    };
    unsafe {
        core::ptr::write_volatile(reg8(0x0080_074c), 0x2c);
        core::ptr::write_volatile(reg32(0x0080_0754), wake_tick);
        core::ptr::write_volatile(reg8(0x0080_074f), 0x08);
        vendor_clock_dly(10);
        vendor_clock_dly(6);
        while (core::ptr::read_volatile(reg8(0x0080_074f).cast_const()) & 0x08) != 0 {}
        core::ptr::write_volatile(reg8(0x0080_074c), 0x20);
    }
    // Keep the gate-to-sleep sequence vendor-tight: extra volatile stores here
    // were enough to perturb PM entry on real hardware.
    if wake44_gate_ready(analog::read(0x44)) {
        sleep_start();
    }
    if sleep_mode_u8 == 0x80 {
        analog::write(0x3c, analog::read(0x3c) & !0x03);
        soft_reboot_dly13ms_use24mRC();
        unsafe { core::ptr::write_volatile(reg8(REG_PWDN_CTRL), 0x20) };
    }

    let now32 = pm_get_32k_tick();
    unsafe {
        let cur = core::ptr::read_volatile(&raw const startup::tick_cur);
        let cur32 = core::ptr::read_volatile(&raw const startup::tick_32k_cur);
        let long = core::ptr::read_volatile(&raw const startup::pm_long_suspend) != 0;
        let upd = if long {
            cur.wrapping_add(
                now32
                    .wrapping_sub(cur32)
                    .wrapping_div(32)
                    .wrapping_mul(CRYSTAL32768_TICK_PER_32CYCLE),
            )
        } else {
            cur.wrapping_add(
                now32
                    .wrapping_sub(cur32)
                    .wrapping_mul(CRYSTAL32768_TICK_PER_32CYCLE)
                    .wrapping_div(32),
            )
        };
        core::ptr::write_volatile(&raw mut startup::tick_cur, upd);
        core::ptr::write_volatile(&raw mut startup::tick_32k_cur, upd.wrapping_add(20 * 16));
        core::ptr::write_volatile(reg8(0x0080_074c), 0x00);
        vendor_clock_dly(7);
        core::ptr::write_volatile(reg8(0x0080_074c), 0x92);
        vendor_clock_dly(4);
        core::ptr::write_volatile(reg8(0x0080_074f), 0x01);
    }
    pm_wait_xtal_ready();
    unsafe { core::ptr::write_volatile(reg8(0x0080_0066), bak66) };
    let st = analog::read(0x44);
    if (st & PM_WAKEUP_COMPARATOR_BITS) != 0 && timer_wakeup {
        loop {
            let now = unsafe { core::ptr::read_volatile(reg32(0x0080_0740).cast_const()) };
            if now.wrapping_sub(wakeup_tick) <= (1u32 << 30) {
                break;
            }
        }
    }
    interrupt::restore(irq_enabled);
    if st != 0 {
        (st as u32 | STATUS_ENTER_SUSPEND) as i32
    } else {
        STATUS_GPIO_ERR_NO_ENTER_PM as i32
    }
}

#[cfg(feature = "chip-8258")]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.pm_long_sleep_wakeup")]
pub extern "C" fn pm_long_sleep_wakeup(
    mode: u32,
    wakeup_src: u32,
    wakeup_duration_ticks_32k: u32,
) -> i32 {
    let sleep_mode = decode_mode(mode);
    let wakeup_src_u8 = wakeup_src as u8;
    let irq_enabled = interrupt::disable();
    let start_tick = unsafe { core::ptr::read_volatile(reg32(0x0080_0740).cast_const()) };
    let calib_reg = unsafe { core::ptr::read_volatile(reg16(0x0080_0750).cast_const()) };
    // VENDOR-DIFF: vendor code assumes `reg16(0x750)` is already non-zero when
    // long sleep is entered. On current hardware/port this register has been
    // observed as zero in the failing RC path, which makes the next divide
    // hang. Keep the vendor read first, but fall back to the already selected
    // 32k-source calibration only for the zero case.
    let calib = if calib_reg != 0 {
        calib_reg
    } else {
        unsafe { core::ptr::read_volatile(&raw const startup::tick_32k_calib) }
    };
    unsafe {
        startup::tick_32k_calib = calib;
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
    let sleep_mode_retention = sleep_mode_u8 & 0x7f;
    let (an7, v2c_base) = if sleep_mode_retention != 0 {
        let t2 = analog::read(0x02);
        analog::write(0x02, (t2 & !0x07) | 0x05);
        unsafe { core::ptr::write_volatile(reg8(0x0080_063e), startup::tl_multi_addr) };
        analog::write(0x2b, 0xde);
        (1u8, 0x56u8)
    } else if sleep_mode_u8 == 0 {
        analog::write(0x04, 0x48);
        analog::write(0x7e, 0x00);
        analog::write(0x2b, 0x5e);
        (4u8, 0x96u8)
    } else {
        analog::write(0x2b, 0xde);
        (5u8, 0xd6u8)
    };
    analog::write(0x7e, sleep_mode_u8);
    let cmp = ((wakeup_src_u32 & PM_WAKEUP_COMPARATOR_BITS as u32) != 0) as u8;
    let any = cmp | (has_timer as u8);
    let mut analog2c = v2c_base | any | (cmp << 3);
    if sleep_mode_retention != 0 && wakeup_src_u8 == PM_WAKEUP_TIMER_BITS {
        analog2c = 0x5e;
    } else if sleep_mode_u8 == SleepMode::DeepSleep.raw() && wakeup_src_u8 == PM_WAKEUP_TIMER_BITS {
        analog2c = 0xde;
    }
    analog::write(0x2c, analog2c);
    analog::write(0x07, (analog::read(0x07) & !0x07) | an7);
    if sleep_mode_retention == 0 {
        unsafe {
            core::ptr::write_volatile(reg8(0x0080_0602), 0x08);
        }
        analog::write(0x7f, 0x01);
    } else {
        analog::write(0x7f, 0x00);
    }
    unsafe {
        let calib_u32 = calib as u32;
        let half = (calib >> 1) as u32;
        if sleep_mode_u8 == SleepMode::DeepSleep.raw() {
            analog::write(0x3c, analog::read(0x3c) | 0x02);
        }
        analog::write(
            0x20,
            0x7f_u32.wrapping_sub((0xfa00_u32.wrapping_add(half)) / calib_u32) as u8,
        );
        let sr =
            core::ptr::read_volatile(&raw const startup::g_pm_r_delay_us.suspend_ret_r_delay_us)
                as u32;
        analog::write(0x1f, !(((sr << 7).wrapping_add(half)) / calib_u32) as u8);
        let dt = core::ptr::read_volatile(reg32(0x0080_0740).cast_const()).wrapping_sub(start_tick);
        let wake_tick = if startup::pm_long_suspend != 0 {
            minus64
                .wrapping_add(startup::tick_32k_cur)
                .wrapping_sub((dt / calib_u32) << 4)
        } else {
            minus64
                .wrapping_add(startup::tick_32k_cur)
                .wrapping_sub((dt << 4).wrapping_add((calib >> 1) as u32) / calib_u32)
        };
        core::ptr::write_volatile(reg8(0x0080_074c), 0x2c);
        core::ptr::write_volatile(reg32(0x0080_0754), wake_tick);
        core::ptr::write_volatile(reg8(0x0080_074f), 0x08);
        vendor_clock_dly(10);
        vendor_clock_dly(6);
        while (core::ptr::read_volatile(reg8(0x0080_074f).cast_const()) & 0x08) != 0 {}
        core::ptr::write_volatile(reg8(0x0080_074c), 0x20);
    }
    // Keep the gate-to-sleep sequence vendor-tight: extra volatile stores here
    // were enough to perturb PM entry on real hardware.
    if wake44_gate_ready(analog::read(0x44)) {
        sleep_start();
    }
    if sleep_mode_u8 == SleepMode::DeepSleep.raw() {
        analog::write(0x3c, analog::read(0x3c) & !0x02);
        soft_reboot_dly13ms_use24mRC();
        unsafe { core::ptr::write_volatile(reg8(REG_PWDN_CTRL), 0x20) };
    }
    unsafe {
        let t32 = pm_get_32k_tick();
        if startup::pm_long_suspend != 0 {
            startup::tick_cur = startup::tick_cur.wrapping_add(
                ((t32.wrapping_sub(startup::tick_32k_cur)) >> 4)
                    .wrapping_mul(startup::tick_32k_calib as u32),
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
        vendor_clock_dly(6);
        core::ptr::write_volatile(reg8(0x0080_074c), 0x90);
        vendor_clock_dly(4);
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
#[unsafe(link_section = ".text.cpu_long_sleep_wakeup_32k_xtal")]
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
    let (an7, mode2c) = if sleep_mode_no_ret != 0 {
        let t2 = analog::read(0x02);
        analog::write(0x02, (t2 & !0x07) | 0x05);
        unsafe { core::ptr::write_volatile(reg8(0x0080_063e), startup::tl_multi_addr) };
        let d = sleep_mode_u8.wrapping_sub(0x80);
        sleep_mode_u8 = (d | (0u8.wrapping_sub(d))) & 0xff;
        (5u8, 0xdeu8)
    } else if sleep_mode_u8 == 0 {
        analog::write(0x04, 0x48);
        analog::write(0x7e, 0x00);
        analog::write(0x2b, 0x5e);
        (4u8, 0x1du8)
    } else {
        analog::write(0x7e, 0x80);
        analog::write(0x2b, 0xde);
        sleep_mode_no_ret = 1;
        if !has_timer {
            let ab = u8::from(irq_enabled);
            switch_ext32kpad_to_int32krc(ab | 0xc0 | (ab << 3));
            (5u8, 0x00u8)
        } else {
            (5u8, 0xc0u8)
        }
    };
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
        let sr =
            core::ptr::read_volatile(&raw const startup::g_pm_r_delay_us.suspend_ret_r_delay_us)
                as u32;
        analog::write(
            0x1f,
            (((sr << 8) + (CRYSTAL32768_TICK_PER_32CYCLE >> 1)) / CRYSTAL32768_TICK_PER_32CYCLE)
                as u8,
        );
        let dt = core::ptr::read_volatile(reg32(0x0080_0740).cast_const()).wrapping_sub(start);
        let wake_tick = if startup::pm_long_suspend != 0 {
            wake_m64
                .wrapping_add(startup::tick_cur)
                .wrapping_sub((dt / CRYSTAL32768_TICK_PER_32CYCLE) << 5)
        } else {
            wake_m64.wrapping_add(startup::tick_cur).wrapping_sub(
                (dt << 5).wrapping_add(CRYSTAL32768_TICK_PER_32CYCLE >> 1)
                    / CRYSTAL32768_TICK_PER_32CYCLE,
            )
        };
        core::ptr::write_volatile(reg8(0x0080_074c), 0x2c);
        core::ptr::write_volatile(reg32(0x0080_0754), wake_tick);
        core::ptr::write_volatile(reg8(0x0080_074f), 0x08);
        vendor_clock_dly(10);
        vendor_clock_dly(6);
        while (core::ptr::read_volatile(reg8(0x0080_074f).cast_const()) & 0x08) != 0 {}
        core::ptr::write_volatile(reg8(0x0080_074c), 0x20);
    }
    // Keep the gate-to-sleep sequence vendor-tight: extra volatile stores here
    // were enough to perturb PM entry on real hardware.
    if wake44_gate_ready(analog::read(0x44)) {
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
                d.wrapping_div(32)
                    .wrapping_mul(CRYSTAL32768_TICK_PER_32CYCLE),
            );
        } else {
            startup::tick_cur = startup::tick_cur.wrapping_add(
                d.wrapping_mul(CRYSTAL32768_TICK_PER_32CYCLE)
                    .wrapping_div(32),
            );
        }
        startup::tick_32k_cur = startup::tick_cur.wrapping_add(20 * 16);
        core::ptr::write_volatile(reg8(0x0080_074c), 0x00);
        vendor_clock_dly(6);
        core::ptr::write_volatile(reg8(0x0080_074c), 0x90);
        vendor_clock_dly(5);
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
#[unsafe(link_section = ".text.check_32k_clk_stable")]
pub extern "C" fn check_32k_clk_stable() {
    startup::startup_pm_wait_xtal_ready();
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.pm_get_32k_tick")]
pub extern "C" fn pm_get_32k_tick() -> u32 {
    startup::startup_pm_get_32k_tick()
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.pm_wait_xtal_ready")]
pub extern "C" fn pm_wait_xtal_ready() {
    startup::startup_pm_wait_xtal_ready();
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.start_reboot")]
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
#[unsafe(link_section = ".text.cpu_stall")]
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
#[unsafe(link_section = ".text.cpu_wakeup_init")]
pub extern "C" fn cpu_wakeup_init() {
    startup::startup_cpu_wakeup_init();
}

#[cfg(test)]
mod tests {
    use super::{
        sleep_ms_kind, sys_ticks_to_32k_ticks, ticks_32k_to_sys_ticks, Clock32kSource, SleepMsKind,
        WakeupSource, PM_NORMAL_SLEEP_MAX_MS,
    };

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

    #[test]
    fn sleep_ms_uses_short_path_at_threshold_and_long_above() {
        assert_eq!(sleep_ms_kind(PM_NORMAL_SLEEP_MAX_MS), SleepMsKind::Short);
        assert_eq!(
            sleep_ms_kind(PM_NORMAL_SLEEP_MAX_MS.saturating_add(1)),
            SleepMsKind::Long
        );
    }

    #[test]
    fn wakeup_source_can_drop_timer_for_pad_only_sleep() {
        let src = (WakeupSource::PAD | WakeupSource::TIMER).without(WakeupSource::TIMER);
        assert_eq!(src, WakeupSource::PAD);
        assert!(!src.contains(WakeupSource::TIMER));
    }
}
