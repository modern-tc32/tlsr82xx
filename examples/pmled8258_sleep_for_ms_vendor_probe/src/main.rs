#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::{OutputPin, PinState};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::{analog, clock, interrupt, pac, pm, startup, timer};

mod platform;

const SLEEP_MS: u32 = 2_000;

const LONG_PULSE_US: u32 = 240_000;
const SHORT_PULSE_US: u32 = 130_000;
const SERIES_GAP_US: u32 = 500_000;
const PRE_SLEEP_GAP_US: u32 = 1_000_000;
const FIRST_START_MARK_US: u32 = 3_000_000;
const EXT32K_SETTLE_US: u32 = 1_500_000;
const RC32K_SETTLE_US: u32 = 30_000;
const POST_WAKE_MARK_US: u32 = 300_000;
const START_MARK_SPIN: u32 = 6_000_000;
const START_GAP_SPIN: u32 = 1_200_000;
const EXT32K_SETTLE_SPIN: u32 = 3_000_000;
const RC32K_SETTLE_SPIN: u32 = 120_000;

#[derive(Clone, Copy)]
struct TestCase {
    clock: pm::Clock32kSource,
    mode: pm::SleepMode,
}

const TESTS: [TestCase; 4] = [
    TestCase {
        clock: pm::Clock32kSource::ExternalCrystal,
        mode: pm::SleepMode::Suspend,
    },
    TestCase {
        clock: pm::Clock32kSource::ExternalCrystal,
        mode: pm::SleepMode::Suspend,
    },
    TestCase {
        clock: pm::Clock32kSource::ExternalCrystal,
        mode: pm::SleepMode::Suspend,
    },
    TestCase {
        clock: pm::Clock32kSource::ExternalCrystal,
        mode: pm::SleepMode::Suspend,
    },
];

#[unsafe(no_mangle)]
static mut NEXT_TEST_INDEX: u8 = 0;
#[unsafe(no_mangle)]
static mut WAS_INITIALIZED: u8 = 0;
#[unsafe(no_mangle)]
static mut ACTIVE_CLOCK_RAW: u8 = 0;
#[unsafe(no_mangle)]
static mut LAST_STEP_RAW: u8 = 0;
#[unsafe(no_mangle)]
static mut DBG_STAGE: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_LAST_RET: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_LOOP_CNT: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_LAST_MODE: u8 = 0;
#[unsafe(no_mangle)]
static mut DBG_LAST_CLOCK: u8 = 0;
#[unsafe(no_mangle)]
static mut DBG_LAST_WAKE_FLAG: u8 = 0;
#[unsafe(no_mangle)]
static mut DBG_ANA44_PRE: u8 = 0;
#[unsafe(no_mangle)]
static mut DBG_NOW_TICK: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_TICKS_PER_US: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_WAKEUP_TICK: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_DELTA_TICK: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_CNT_BOOT: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_CNT_BEFORE_SLEEP: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_CNT_ENTER_SLEEP: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_CNT_AFTER_SLEEP: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_CNT_LED_W_ON: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_CNT_LED_W_OFF: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_CNT_LED_Y_ON: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_CNT_LED_Y_OFF: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_CNT_EARLY_LED_OFF: u32 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    early_force_leds_off();
    dbg_inc(&raw mut DBG_CNT_BOOT);
    dbg_u32(&raw mut DBG_STAGE, 0x01);
    let _ = platform::init();
    dbg_u32(&raw mut DBG_STAGE, 0x02);
    clock::init(clock::SysClock::Crystal16M);
    pm::sync_sys_tick_per_us();
    pm::init(pm::Clock32kSource::ExternalCrystal);
    unsafe {
        ACTIVE_CLOCK_RAW = 2;
    }
    let _ = interrupt::enable();

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
    drive_white(&mut board, false);
    drive_yellow(&mut board, false);

    if unsafe { WAS_INITIALIZED } == 0 {
        unsafe {
            WAS_INITIALIZED = 1;
            NEXT_TEST_INDEX = 0;
        }
        drive_white(&mut board, false);
        drive_yellow(&mut board, false);
    }

    let mut idx = unsafe { NEXT_TEST_INDEX as usize % TESTS.len() };
    loop {
        dbg_u32(&raw mut DBG_STAGE, 0x10);
        dbg_inc(&raw mut DBG_LOOP_CNT);
        clock::init(clock::SysClock::Crystal16M);
        pm::sync_sys_tick_per_us();
        let _ = interrupt::enable();
        board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
        drive_white(&mut board, false);
        drive_yellow(&mut board, false);

        let case = TESTS[idx];
        unsafe {
            LAST_STEP_RAW = idx as u8;
            core::ptr::write_volatile(&raw mut NEXT_TEST_INDEX, ((idx + 1) % TESTS.len()) as u8);
            core::ptr::write_volatile(&raw mut DBG_LAST_MODE, case.mode.raw());
            core::ptr::write_volatile(
                &raw mut DBG_LAST_CLOCK,
                match case.clock {
                    pm::Clock32kSource::InternalRc => 1,
                    pm::Clock32kSource::ExternalCrystal => 2,
                },
            );
            core::ptr::write_volatile(
                &raw mut DBG_LAST_WAKE_FLAG,
                startup::PM_STARTUP_DBG_WAKEUP_FLAG,
            );
        }

        // Marker format: 3 white + N yellow (N=1..4).
        blink_n_white(&mut board, 3, 220_000);
        delay_us(300_000);
        blink_n_yellow(&mut board, (idx as u8).wrapping_add(1), 220_000, 320_000);
        delay_us(500_000);

        dbg_u32(&raw mut DBG_STAGE, 0x20);
        switch_32k_if_needed(case.clock);
        dbg_u32(&raw mut DBG_STAGE, 0x30);
        dbg_inc(&raw mut DBG_CNT_BEFORE_SLEEP);

        unsafe {
            core::ptr::write_volatile(&raw mut DBG_ANA44_PRE, analog::read(0x44));
        }
        let _ = sleep_for_ms_vendor(case.mode, case.clock, pm::WakeupSource::TIMER, SLEEP_MS);
        dbg_u32(&raw mut DBG_STAGE, 0x40);
        dbg_inc(&raw mut DBG_CNT_AFTER_SLEEP);
        idx = (idx + 1) % TESTS.len();
    }
}

#[inline(always)]
fn switch_32k_if_needed(source: pm::Clock32kSource) {
    match source {
        pm::Clock32kSource::InternalRc => {
            if unsafe { ACTIVE_CLOCK_RAW } != 1 {
                pm::init(pm::Clock32kSource::InternalRc);
                spin_delay(RC32K_SETTLE_SPIN);
                unsafe {
                    ACTIVE_CLOCK_RAW = 1;
                }
            }
        }
        pm::Clock32kSource::ExternalCrystal => {
            if unsafe { ACTIVE_CLOCK_RAW } != 2 {
                pm::init(pm::Clock32kSource::ExternalCrystal);
                spin_delay(EXT32K_SETTLE_SPIN);
                unsafe {
                    ACTIVE_CLOCK_RAW = 2;
                }
            }
        }
    }
}

#[inline(always)]
fn sleep_for_ms_vendor(
    mode: pm::SleepMode,
    source: pm::Clock32kSource,
    wakeup_src: pm::WakeupSource,
    duration_ms: u32,
) -> u32 {
    let _ = source;
    dbg_u32(&raw mut DBG_STAGE, 0x31);
    let now = timer::clock_time();
    let tpu = timer::sys_tick_per_us();
    let wakeup_tick = now.wrapping_add(duration_ms.saturating_mul(1000).saturating_mul(tpu));
    startup::set_tick_cur(now);
    startup::set_tick_32k_cur(startup::pm_get_32k_tick());
    startup::set_pm_long_suspend(false);
    dbg_u32(&raw mut DBG_NOW_TICK, now);
    dbg_u32(&raw mut DBG_TICKS_PER_US, tpu);
    dbg_u32(&raw mut DBG_WAKEUP_TICK, wakeup_tick);
    dbg_u32(&raw mut DBG_DELTA_TICK, wakeup_tick.wrapping_sub(now));
    dbg_u32(&raw mut DBG_STAGE, 0x32);
    dbg_inc(&raw mut DBG_CNT_ENTER_SLEEP);
    let ret = if mode == pm::SleepMode::Suspend {
        // Suspend diagnostic path: bypass PM deep sleep handler and use cpu_stall timer wakeup.
        // This isolates whether base timer wakeup is functional in current environment.
        let stall_mask = if wakeup_src.contains(pm::WakeupSource::TIMER) {
            0x02
        } else {
            0
        };
        let mut remain_us = duration_ms.saturating_mul(1000);
        let mut stall_ret = 0u32;
        while remain_us != 0 {
            let slice_us = if remain_us > 900_000 {
                900_000
            } else {
                remain_us
            };
            stall_ret = startup::cpu_stall(stall_mask, slice_us, tpu);
            remain_us = remain_us.saturating_sub(slice_us);
        }
        stall_ret
    } else {
        unsafe {
            let handler = core::ptr::read_volatile(&raw const startup::cpu_sleep_wakeup);
            let cpu_sleep_wakeup: unsafe extern "C" fn(u32, u32, u32) -> i32 =
                core::mem::transmute(handler);
            cpu_sleep_wakeup(mode.raw() as u32, wakeup_src.raw() as u32, wakeup_tick) as u32
        }
    };
    dbg_u32(&raw mut DBG_LAST_RET, ret);
    dbg_u32(&raw mut DBG_STAGE, 0x33);
    ret
}

#[inline(always)]
fn dbg_u32(slot: *mut u32, value: u32) {
    unsafe {
        core::ptr::write_volatile(slot, value);
    }
}

#[inline(always)]
fn dbg_inc(slot: *mut u32) {
    unsafe {
        let v = core::ptr::read_volatile(slot.cast_const());
        core::ptr::write_volatile(slot, v.wrapping_add(1));
    }
}

#[inline(always)]
fn dbg_u32_read(slot: *const u32) -> u32 {
    unsafe { core::ptr::read_volatile(slot) }
}

fn indicate_wake_class(board: &mut Board) {
    let count = match pm::wake_origin() {
        pm::WakeOrigin::ColdBoot => 1,
        pm::WakeOrigin::DeepWake => 2,
        pm::WakeOrigin::DeepRetentionWake => 3,
    };
    blink_n_white(board, count, LONG_PULSE_US);
}

fn indicate_step(board: &mut Board) {
    delay_us(SERIES_GAP_US);
    let count = unsafe { LAST_STEP_RAW.wrapping_add(1) };
    let count = if count == 0 { 1 } else { count };
    blink_n_yellow(board, count, SHORT_PULSE_US, SHORT_PULSE_US.saturating_mul(2));
}

fn drive_white(board: &mut Board, on: bool) {
    if on {
        dbg_inc(&raw mut DBG_CNT_LED_W_ON);
    } else {
        dbg_inc(&raw mut DBG_CNT_LED_W_OFF);
    }
    let _ = board.led_w.set_state(PinState::from(on));
}

fn drive_yellow(board: &mut Board, on: bool) {
    if on {
        dbg_inc(&raw mut DBG_CNT_LED_Y_ON);
    } else {
        dbg_inc(&raw mut DBG_CNT_LED_Y_OFF);
    }
    let _ = board.led_y.set_state(PinState::from(on));
}

fn delay_us(duration_us: u32) {
    let started = timer::clock_time();
    let mut guard = 0u32;
    while !timer::clock_time_exceed_us(started, duration_us) {
        core::hint::spin_loop();
        guard = guard.wrapping_add(1);
        if guard >= 3_000_000 {
            break;
        }
    }
}

#[inline(always)]
fn spin_delay(iter: u32) {
    let mut i = 0u32;
    while i < iter {
        core::hint::spin_loop();
        i = i.wrapping_add(1);
    }
}

#[inline(always)]
fn early_force_leds_off() {
    const REG_GPIOB_IE: usize = 0x0080_0589;
    const REG_GPIOB_OEN: usize = 0x0080_058a; // active-low
    const REG_GPIOB_OUT: usize = 0x0080_058b;
    const REG_GPIOB_FUNC: usize = 0x0080_058e;
    const LED_Y_MASK: u8 = 1 << 4; // PB4
    const LED_W_MASK: u8 = 1 << 5; // PB5
    const LED_MASK: u8 = LED_Y_MASK | LED_W_MASK;

    unsafe {
        let func = (REG_GPIOB_FUNC as *const u8).read_volatile();
        (REG_GPIOB_FUNC as *mut u8).write_volatile(func | LED_MASK);

        let ie = (REG_GPIOB_IE as *const u8).read_volatile();
        (REG_GPIOB_IE as *mut u8).write_volatile(ie & !LED_MASK);

        let out = (REG_GPIOB_OUT as *const u8).read_volatile();
        (REG_GPIOB_OUT as *mut u8).write_volatile(out & !LED_MASK);

        let oen = (REG_GPIOB_OEN as *const u8).read_volatile();
        (REG_GPIOB_OEN as *mut u8).write_volatile(oen & !LED_MASK);
    }

    dbg_inc(&raw mut DBG_CNT_EARLY_LED_OFF);
}

fn blink_n_white(board: &mut Board, count: u8, pulse_us: u32) {
    blink_n_custom(board, LedColor::White, count, pulse_us, pulse_us);
}

fn blink_n_yellow(board: &mut Board, count: u8, on_us: u32, off_us: u32) {
    blink_n_custom(board, LedColor::Yellow, count, on_us, off_us);
}

#[derive(Clone, Copy)]
enum LedColor {
    White,
    Yellow,
}

fn blink_n_custom(board: &mut Board, color: LedColor, count: u8, on_us: u32, off_us: u32) {
    let mut i = 0u8;
    while i < count {
        match color {
            LedColor::White => drive_white(board, true),
            LedColor::Yellow => drive_yellow(board, true),
        }
        delay_us(on_us);
        match color {
            LedColor::White => drive_white(board, false),
            LedColor::Yellow => drive_yellow(board, false),
        }
        delay_us(off_us);
        i = i.wrapping_add(1);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
