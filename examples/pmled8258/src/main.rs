#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::{OutputPin, PinState};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::{clock, interrupt, pac, pm, startup, timer};

mod platform;

const SLEEP_MS: u32 = 2_000;
const WAKE_BLINK_US: u32 = 260_000;
const MODE_BLINK_US: u32 = 360_000;
const STARTUP_BLINK_US: u32 = 220_000;
const PM_DIAG_MAGIC_VALUE: u32 = 0x504D_4447;

#[unsafe(no_mangle)]
static mut PM_DIAG_MAGIC: u32 = 0;
#[unsafe(no_mangle)]
static mut PM_DIAG_BOOT_COUNT: u32 = 0;
#[unsafe(no_mangle)]
static mut PM_DIAG_WAKE_COUNT: u32 = 0;
#[unsafe(no_mangle)]
static mut PM_DIAG_LOOP_COUNT: u32 = 0;
#[unsafe(no_mangle)]
static mut PM_DIAG_WAKE_ORIGIN: u32 = 0;
#[unsafe(no_mangle)]
static mut PM_DIAG_WAKE_SRC_RAW: u32 = 0;
#[unsafe(no_mangle)]
static mut PM_DIAG_LAST_SLEEP_MODE: u32 = 0;
#[unsafe(no_mangle)]
static mut PM_DIAG_NEXT_MODE: u32 = 0;
#[unsafe(no_mangle)]
static mut PM_DIAG_STARTUP_WAKEUP_FLAG: u32 = 0;
#[unsafe(no_mangle)]
static mut PM_DIAG_STARTUP_ANA7F: u32 = 0;
#[unsafe(no_mangle)]
static mut PM_DIAG_STARTUP_ANA3C: u32 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();
    clock::init(clock::SysClock::Crystal16M);
    pm::sync_sys_tick_per_us();
    // pm::init(pm::Clock32kSource::ExternalCrystal);
    pm::init(pm::Clock32kSource::InternalRc);
    let _ = interrupt::enable();
    diag_record_startup();

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
    drive_pin(&mut board.led_y, false);
    drive_pin(&mut board.led_w, false);
    blink_startup_debug(&mut board);

    loop {
        // Mark each wake with a short yellow pulse.
        drive_pin(&mut board.led_y, true);
        delay_us(WAKE_BLINK_US);
        drive_pin(&mut board.led_y, false);
        let mode_blinks = last_mode_blink_count();
        blink_n(&mut board.led_y, mode_blinks, MODE_BLINK_US);

        let mode = pm::SleepMode::DeepSleep;
        diag_before_sleep(mode);

        let _ = pm::sleep_for_ms(mode, pm::WakeupSource::TIMER, SLEEP_MS);
    }
}

fn drive_pin<P: OutputPin>(pin: &mut P, high: bool) {
    let _ = pin.set_state(PinState::from(high));
}

fn delay_us(duration_us: u32) {
    let started = timer::clock_time();
    while !timer::clock_time_exceed_us(started, duration_us) {
        core::hint::spin_loop();
    }
}

fn blink_n<P: OutputPin>(pin: &mut P, count: u8, pulse_us: u32) {
    let mut i = 0u8;
    while i < count {
        drive_pin(pin, true);
        delay_us(pulse_us);
        drive_pin(pin, false);
        delay_us(pulse_us);
        i = i.wrapping_add(1);
    }
}

fn blink_startup_debug(board: &mut Board) {
    let wakeup_flag = unsafe { startup::PM_STARTUP_DBG_WAKEUP_FLAG };
    let ana7f = unsafe { startup::PM_STARTUP_DBG_ANA7F };
    let ana3c = unsafe { startup::PM_STARTUP_DBG_ANA3C };

    let white_count = match wakeup_flag {
        0 => 1,
        1 => 2,
        2 => 3,
        _ => 4,
    };
    blink_n(&mut board.led_w, white_count, STARTUP_BLINK_US);

    if (ana7f & 0x01) != 0 {
        blink_n(&mut board.led_y, 1, STARTUP_BLINK_US);
    }
    if (ana3c & 0x02) != 0 {
        blink_n(&mut board.led_y, 2, STARTUP_BLINK_US);
    }
}

fn last_mode_blink_count() -> u8 {
    let last = unsafe { core::ptr::read_volatile(&raw const PM_DIAG_LAST_SLEEP_MODE) as u8 };
    if last == pm::SleepMode::DeepSleep as u8 {
        2
    } else if last == pm::SleepMode::DeepSleepRetentionLow16K as u8 {
        4
    } else if last == pm::SleepMode::DeepSleepRetentionLow32K as u8 {
        5
    } else {
        3
    }
}

#[inline(always)]
fn diag_record_startup() {
    unsafe {
        if core::ptr::read_volatile(&raw const PM_DIAG_MAGIC) != PM_DIAG_MAGIC_VALUE {
            core::ptr::write_volatile(&raw mut PM_DIAG_MAGIC, PM_DIAG_MAGIC_VALUE);
            core::ptr::write_volatile(&raw mut PM_DIAG_BOOT_COUNT, 0);
            core::ptr::write_volatile(&raw mut PM_DIAG_WAKE_COUNT, 0);
            core::ptr::write_volatile(&raw mut PM_DIAG_LOOP_COUNT, 0);
            core::ptr::write_volatile(&raw mut PM_DIAG_WAKE_ORIGIN, 0);
            core::ptr::write_volatile(&raw mut PM_DIAG_WAKE_SRC_RAW, 0);
            core::ptr::write_volatile(&raw mut PM_DIAG_LAST_SLEEP_MODE, 0);
            core::ptr::write_volatile(&raw mut PM_DIAG_NEXT_MODE, 0);
            core::ptr::write_volatile(&raw mut PM_DIAG_STARTUP_WAKEUP_FLAG, 0);
            core::ptr::write_volatile(&raw mut PM_DIAG_STARTUP_ANA7F, 0);
            core::ptr::write_volatile(&raw mut PM_DIAG_STARTUP_ANA3C, 0);
        }

        if pm::is_cold_boot() {
            let count = core::ptr::read_volatile(&raw const PM_DIAG_BOOT_COUNT).wrapping_add(1);
            core::ptr::write_volatile(&raw mut PM_DIAG_BOOT_COUNT, count);
        } else {
            let count = core::ptr::read_volatile(&raw const PM_DIAG_WAKE_COUNT).wrapping_add(1);
            core::ptr::write_volatile(&raw mut PM_DIAG_WAKE_COUNT, count);
        }

        let origin = match pm::wake_origin() {
            pm::WakeOrigin::ColdBoot => 0u32,
            pm::WakeOrigin::DeepWake => 1u32,
            pm::WakeOrigin::DeepRetentionWake => 2u32,
        };
        core::ptr::write_volatile(&raw mut PM_DIAG_WAKE_ORIGIN, origin);
        core::ptr::write_volatile(&raw mut PM_DIAG_WAKE_SRC_RAW, pm::wakeup_source_raw() as u32);
        core::ptr::write_volatile(
            &raw mut PM_DIAG_STARTUP_WAKEUP_FLAG,
            startup::PM_STARTUP_DBG_WAKEUP_FLAG as u32,
        );
        core::ptr::write_volatile(&raw mut PM_DIAG_STARTUP_ANA7F, startup::PM_STARTUP_DBG_ANA7F as u32);
        core::ptr::write_volatile(&raw mut PM_DIAG_STARTUP_ANA3C, startup::PM_STARTUP_DBG_ANA3C as u32);
    }
}

#[inline(always)]
fn diag_next_mode() -> u32 {
    unsafe { core::ptr::read_volatile(&raw const PM_DIAG_NEXT_MODE) }
}

#[inline(always)]
fn diag_before_sleep(mode: pm::SleepMode) {
    unsafe {
        core::ptr::write_volatile(&raw mut PM_DIAG_LAST_SLEEP_MODE, mode as u32);
        let next = 0;
        core::ptr::write_volatile(&raw mut PM_DIAG_NEXT_MODE, next);
        let loops = core::ptr::read_volatile(&raw const PM_DIAG_LOOP_COUNT).wrapping_add(1);
        core::ptr::write_volatile(&raw mut PM_DIAG_LOOP_COUNT, loops);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
