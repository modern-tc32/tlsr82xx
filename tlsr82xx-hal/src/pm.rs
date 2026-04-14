use crate::{analog, gpio, startup, timer};

#[cfg(feature = "chip-8258")]
use crate::mmio::{reg32, reg8};
#[cfg(feature = "chip-8258")]
use crate::regs8258::{REG_MCU_WAKEUP_MASK, REG_PWDN_CTRL};

#[cfg(feature = "chip-8258")]
const REG_PM_WAIT: usize = 0x0080_074c;
#[cfg(feature = "chip-8258")]
const REG_PM_TICK_CTRL: usize = 0x0080_074f;
#[cfg(feature = "chip-8258")]
const REG_SYSTEM_WAKEUP_TICK: usize = 0x0080_0754;

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

static mut CLOCK_32K_SOURCE: Clock32kSource = Clock32kSource::InternalRc;

#[cfg(all(feature = "chip-8258", feature = "vendor-pm"))]
unsafe extern "C" {
    #[link_name = "cpu_sleep_wakeup_32k_rc"]
    fn vendor_cpu_sleep_wakeup_32k_rc(
        mode: SleepMode,
        wakeup_src: WakeupSource,
        wakeup_tick: u32,
    ) -> i32;
    #[link_name = "cpu_sleep_wakeup_32k_xtal"]
    fn vendor_cpu_sleep_wakeup_32k_xtal(
        mode: SleepMode,
        wakeup_src: WakeupSource,
        wakeup_tick: u32,
    ) -> i32;
    #[link_name = "pm_long_sleep_wakeup"]
    fn vendor_pm_long_sleep_wakeup(
        mode: SleepMode,
        wakeup_src: WakeupSource,
        wakeup_duration_ticks_32k: u32,
    ) -> i32;
}

#[inline(always)]
pub fn init(source: Clock32kSource) {
    #[cfg(feature = "chip-8258")]
    if source == Clock32kSource::InternalRc {
        // Vendor clock_32k_init(0) path: switch 32k mux to internal RC.
        let clk32k_sel = analog::read(0x2d) & 0x7f;
        analog::write(0x2d, clk32k_sel);
        let mut pm32k_ctrl = analog::read(0x05) & !0x03;
        pm32k_ctrl |= 0x02;
        analog::write(0x05, pm32k_ctrl);

        rc_32k_cal_vendor_like();
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
    if mode.is_suspend() {
        return sleep_impl(
            mode,
            wakeup_src,
            wakeup_tick,
            current_32k_source(),
            false,
        ) as u32;
    }
    match current_32k_source() {
        Clock32kSource::InternalRc => cpu_sleep_wakeup_32k_rc_dispatch(mode, wakeup_src, wakeup_tick) as u32,
        Clock32kSource::ExternalCrystal => {
            cpu_sleep_wakeup_32k_xtal_dispatch(mode, wakeup_src, wakeup_tick) as u32
        }
    }
}

#[inline(always)]
pub fn sleep_for_ms(mode: SleepMode, wakeup_src: WakeupSource, duration_ms: u32) -> u32 {
    let ticks_per_us = timer::sys_tick_per_us();
    let wakeup_tick = timer::clock_time().wrapping_add(
        duration_ms
            .saturating_mul(1000)
            .saturating_mul(ticks_per_us),
    );
    sleep_until_tick(mode, wakeup_src, wakeup_tick)
}

#[inline(always)]
pub fn long_sleep_32k(
    mode: SleepMode,
    wakeup_src: WakeupSource,
    duration_ticks_32k: u32,
) -> u32 {
    long_sleep_wakeup_impl(mode, wakeup_src, duration_ticks_32k) as u32
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
        Clock32kSource::InternalRc => cpu_sleep_wakeup_32k_rc_dispatch as *const () as usize,
        Clock32kSource::ExternalCrystal => cpu_sleep_wakeup_32k_xtal_dispatch as *const () as usize,
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
    // Keep analog wake-source config in sync with vendor PM path.
    analog::write(0x26, wakeup_src.raw());
    analog::write(0x44, 0x0f);

    unsafe {
        core::ptr::write_volatile(reg8(REG_MCU_WAKEUP_MASK), wakeup_src.raw());
        if wakeup_src.contains(WakeupSource::TIMER) {
            let program_tick = if long_sleep {
                let delta_32k = wakeup_tick.wrapping_sub(startup::current_tick_32k_cur());
                startup::current_tick_cur().wrapping_add(ticks_32k_to_sys_ticks(delta_32k, source))
            } else {
                wakeup_tick
            };
            core::ptr::write_volatile(reg8(REG_PM_WAIT), 0x2c);
            core::ptr::write_volatile(reg32(REG_SYSTEM_WAKEUP_TICK), program_tick);
            core::ptr::write_volatile(reg8(REG_PM_TICK_CTRL), 0x08);
            while core::ptr::read_volatile(reg8(REG_PM_TICK_CTRL).cast_const()) != 0 {
                core::hint::spin_loop();
            }
            core::ptr::write_volatile(reg8(REG_PM_WAIT), 0x20);
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
        let wake = startup::cpu_stall(stall_mask, interval_us, ticks_per_us);
        return wake | STATUS_ENTER_SUSPEND;
    }

    unsafe {
        core::ptr::write_volatile(reg8(REG_PWDN_CTRL), mode.raw());
    }

    startup::sleep_start();

    let wake_raw = analog::read(0x44);
    let mut wakeup_status = 0u32;
    if (wake_raw & 0x01) != 0 {
        wakeup_status |= WAKEUP_STATUS_COMPARATOR;
    }
    if (wake_raw & 0x02) != 0 {
        wakeup_status |= WAKEUP_STATUS_TIMER;
    }
    if (wake_raw & 0x04) != 0 {
        wakeup_status |= WAKEUP_STATUS_CORE;
    }
    if (wake_raw & 0x08) != 0 {
        wakeup_status |= WAKEUP_STATUS_PAD;
    }
    wakeup_status
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

#[cfg(feature = "chip-8258")]
#[inline(always)]
fn cpu_sleep_wakeup_32k_rc_dispatch(mode: SleepMode, wakeup_src: WakeupSource, wakeup_tick: u32) -> i32 {
    #[cfg(feature = "vendor-pm")]
    unsafe {
        return vendor_cpu_sleep_wakeup_32k_rc(mode, wakeup_src, wakeup_tick);
    }
    #[cfg(not(feature = "vendor-pm"))]
    {
        sleep_impl(mode, wakeup_src, wakeup_tick, Clock32kSource::InternalRc, false)
    }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
fn cpu_sleep_wakeup_32k_xtal_dispatch(
    mode: SleepMode,
    wakeup_src: WakeupSource,
    wakeup_tick: u32,
) -> i32 {
    #[cfg(feature = "vendor-pm")]
    unsafe {
        return vendor_cpu_sleep_wakeup_32k_xtal(mode, wakeup_src, wakeup_tick);
    }
    #[cfg(not(feature = "vendor-pm"))]
    {
        sleep_impl(mode, wakeup_src, wakeup_tick, Clock32kSource::ExternalCrystal, false)
    }
}

#[cfg(all(feature = "chip-8258", not(feature = "vendor-pm")))]
#[unsafe(no_mangle)]
pub extern "C" fn cpu_sleep_wakeup_32k_rc(
    mode: SleepMode,
    wakeup_src: WakeupSource,
    wakeup_tick: u32,
) -> i32 {
    cpu_sleep_wakeup_32k_rc_dispatch(mode, wakeup_src, wakeup_tick)
}

#[cfg(all(feature = "chip-8258", not(feature = "vendor-pm")))]
#[unsafe(no_mangle)]
pub extern "C" fn cpu_sleep_wakeup_32k_xtal(
    mode: SleepMode,
    wakeup_src: WakeupSource,
    wakeup_tick: u32,
) -> i32 {
    cpu_sleep_wakeup_32k_xtal_dispatch(mode, wakeup_src, wakeup_tick)
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
fn long_sleep_wakeup_impl(
    mode: SleepMode,
    wakeup_src: WakeupSource,
    wakeup_duration_ticks_32k: u32,
) -> i32 {
    #[cfg(feature = "vendor-pm")]
    if current_32k_source() == Clock32kSource::InternalRc {
        return unsafe { vendor_pm_long_sleep_wakeup(mode, wakeup_src, wakeup_duration_ticks_32k) };
    }
    let wakeup_tick = current_32k_tick().wrapping_add(wakeup_duration_ticks_32k);
    let source = current_32k_source();
    sleep_impl(mode, wakeup_src, wakeup_tick, source, true)
}

#[cfg(all(feature = "chip-8258", not(feature = "vendor-pm")))]
#[unsafe(no_mangle)]
pub extern "C" fn pm_long_sleep_wakeup(
    mode: SleepMode,
    wakeup_src: WakeupSource,
    wakeup_duration_ticks_32k: u32,
) -> i32 {
    long_sleep_wakeup_impl(mode, wakeup_src, wakeup_duration_ticks_32k)
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
