#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::{OutputPin, PinState};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::interrupt;
use tlsr82xx_hal::pac;
use tlsr82xx_hal::timer;

mod platform;

const IRQ_PERIOD_US: u32 = 200_000;
const IRQ_PERIOD_TICKS: u32 = IRQ_PERIOD_US * timer::SYS_TICK_PER_US;

#[repr(C)]
pub struct SystemTimerTrace {
    boot_tick: u32,
    irq_count: u32,
    led_phase: u32,
}

#[unsafe(no_mangle)]
pub static mut TLSR82XX_SYSTIMER_TRACE: SystemTimerTrace = SystemTimerTrace {
    boot_tick: 0,
    irq_count: 0,
    led_phase: 0,
};

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();
    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
    unsafe {
        core::ptr::write_volatile(&raw mut TLSR82XX_SYSTIMER_TRACE.boot_tick, timer::clock_time());
    }

    timer::configure_system_timer_periodic_irq(IRQ_PERIOD_TICKS);
    interrupt::enable();

    let _ = board.led_y.set_state(PinState::from(false));
    let mut last_phase = false;
    loop {
        let count = timer::system_timer_irq_count();
        let phase = timer::system_timer_irq_phase();
        unsafe {
            core::ptr::write_volatile(&raw mut TLSR82XX_SYSTIMER_TRACE.irq_count, count);
            core::ptr::write_volatile(&raw mut TLSR82XX_SYSTIMER_TRACE.led_phase, phase as u32);
        }
        if phase != last_phase {
            last_phase = phase;
            let _ = board.led_w.set_state(PinState::from(phase));
        }
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
