use crate::{gpio, startup, timer};

#[cfg(feature = "chip-8258")]
use crate::mmio::{reg32, reg8};
#[cfg(feature = "chip-8258")]
use crate::regs8258::{REG_MCU_WAKEUP_MASK, REG_PWDN_CTRL, REG_SYSTEM_TICK, REG_WAKEUP_SRC};

#[cfg(feature = "chip-8258")]
const REG_SYSTEM_WAKEUP_TICK: usize = 0x0080_0748;

const SYS_TICK_HZ: u32 = 16_000_000;
const RC_32K_HZ: u32 = 32_000;
const XTAL_32K_HZ: u32 = 32_768;

const PM_WAKEUP_PAD_BITS: u8 = 1 << 4;
const PM_WAKEUP_CORE_BITS: u8 = 1 << 5;
const PM_WAKEUP_TIMER_BITS: u8 = 1 << 6;
const PM_WAKEUP_COMPARATOR_BITS: u8 = 1 << 7;

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

static mut CLOCK_32K_SOURCE: Clock32kSource = Clock32kSource::InternalRc;

#[inline(always)]
pub fn init(source: Clock32kSource) {
    select_32k_source(source);
}

#[inline(always)]
pub fn state() -> startup::StartupState {
    startup::startup_state()
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
    startup::pm_set_wakeup_time_param(startup::PmRDelayUs {
        deep_r_delay_us: timing.deep_r_delay_us,
        suspend_ret_r_delay_us: timing.suspend_ret_r_delay_us,
    });
}

#[inline(always)]
pub fn set_xtal_stable_timing(timing: XtalStableTiming) {
    startup::pm_set_xtal_stable_timer_param(timing.delay_us, timing.loop_count, timing.nop_count);
}

#[inline(always)]
pub fn configure_gpio_wakeup(raw_pin: gpio::RawPin, level: WakeupLevel, enabled: bool) {
    startup::cpu_set_gpio_wakeup(raw_pin.as_u16() as u32, level as u32, i32::from(enabled));
}

#[inline(always)]
pub fn sleep_until_tick(mode: SleepMode, wakeup_src: WakeupSource, wakeup_tick: u32) -> u32 {
    match current_32k_source() {
        Clock32kSource::InternalRc => cpu_sleep_wakeup_32k_rc(mode, wakeup_src, wakeup_tick) as u32,
        Clock32kSource::ExternalCrystal => {
            cpu_sleep_wakeup_32k_xtal(mode, wakeup_src, wakeup_tick) as u32
        }
    }
}

#[inline(always)]
pub fn sleep_for_ms(mode: SleepMode, wakeup_src: WakeupSource, duration_ms: u32) -> u32 {
    let wakeup_tick = timer::clock_time().wrapping_add(
        duration_ms
            .saturating_mul(1000)
            .saturating_mul(timer::SYS_TICK_PER_US),
    );
    sleep_until_tick(mode, wakeup_src, wakeup_tick)
}

#[inline(always)]
pub fn long_sleep_32k(mode: SleepMode, wakeup_src: WakeupSource, wakeup_ticks_32k: u32) -> u32 {
    pm_long_sleep_wakeup(mode, wakeup_src, wakeup_ticks_32k) as u32
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

    let recover = match source {
        Clock32kSource::InternalRc => pm_tim_recover_32k_rc as *const () as usize,
        Clock32kSource::ExternalCrystal => pm_tim_recover_32k_xtal as *const () as usize,
    };
    let sleep = match source {
        Clock32kSource::InternalRc => cpu_sleep_wakeup_32k_rc as *const () as usize,
        Clock32kSource::ExternalCrystal => cpu_sleep_wakeup_32k_xtal as *const () as usize,
    };

    startup::set_pm_tim_recover_handler(recover);
    startup::set_cpu_sleep_wakeup_handler(sleep);
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
fn current_32k_tick() -> u32 {
    startup::pm_get_32k_tick()
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
fn enter_sleep(mode: SleepMode) -> u32 {
    if mode.is_suspend() {
        startup::sleep_start();
        let wake = unsafe { core::ptr::read_volatile(reg32(REG_WAKEUP_SRC).cast_const()) };
        unsafe {
            core::ptr::write_volatile(reg32(REG_SYSTEM_TICK), timer::clock_time());
            core::ptr::write_volatile(reg8(REG_PWDN_CTRL), 0);
        }
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
    prepare_sleep(wakeup_src, wakeup_tick, source, long_sleep);
    enter_sleep(mode) as i32
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

#[inline(always)]
pub extern "C" fn pm_tim_recover_32k_rc(now_tick_32k: u32) -> u32 {
    pm_tim_recover_impl(now_tick_32k, Clock32kSource::InternalRc)
}

#[inline(always)]
pub extern "C" fn pm_tim_recover_32k_xtal(now_tick_32k: u32) -> u32 {
    pm_tim_recover_impl(now_tick_32k, Clock32kSource::ExternalCrystal)
}

fn pm_tim_recover_impl(now_tick_32k: u32, source: Clock32kSource) -> u32 {
    let prev_32k = startup::current_tick_32k_cur();
    let prev_sys = startup::current_tick_cur();
    let delta_32k = now_tick_32k.wrapping_sub(prev_32k);
    let recovered = prev_sys.wrapping_add(ticks_32k_to_sys_ticks(delta_32k, source));
    startup::set_tick_32k_cur(now_tick_32k);
    startup::set_tick_cur(recovered);
    recovered
}

#[inline(always)]
pub extern "C" fn cpu_sleep_wakeup_32k_rc(
    mode: SleepMode,
    wakeup_src: WakeupSource,
    wakeup_tick: u32,
) -> i32 {
    sleep_impl(
        mode,
        wakeup_src,
        wakeup_tick,
        Clock32kSource::InternalRc,
        false,
    )
}

#[inline(always)]
pub extern "C" fn cpu_sleep_wakeup_32k_xtal(
    mode: SleepMode,
    wakeup_src: WakeupSource,
    wakeup_tick: u32,
) -> i32 {
    sleep_impl(
        mode,
        wakeup_src,
        wakeup_tick,
        Clock32kSource::ExternalCrystal,
        false,
    )
}

#[inline(always)]
pub extern "C" fn pm_long_sleep_wakeup(
    mode: SleepMode,
    wakeup_src: WakeupSource,
    wakeup_tick: u32,
) -> i32 {
    let source = current_32k_source();
    sleep_impl(mode, wakeup_src, wakeup_tick, source, true)
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
