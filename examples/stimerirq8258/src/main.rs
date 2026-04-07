#![no_std]
#![no_main]

use core::cell::UnsafeCell;
use core::panic::PanicInfo;

use embedded_hal::digital::{OutputPin, PinState};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::interrupt::{self, Irq};
use tlsr82xx_hal::pac;
use tlsr82xx_hal::timer;

mod platform;

const IRQ_PERIOD_US: u32 = 250_000;
const TIMER0_PERIOD_TICKS: u32 = IRQ_PERIOD_US * timer::SYS_TICK_PER_US;

struct SharedCounter(UnsafeCell<u32>);

unsafe impl Sync for SharedCounter {}

static IRQ_COUNT: SharedCounter = SharedCounter(UnsafeCell::new(0));

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });

    timer::clear_timer0_status();
    interrupt::clear_irq(Irq::Timer0);
    timer::set_timer0_mode_sysclk();
    timer::set_timer0_irq_capture(TIMER0_PERIOD_TICKS);
    timer::enable_timer0();
    interrupt::register_global_irq_handler(timer0_irq_handler);
    interrupt::enable_irq(Irq::Timer0);
    interrupt::enable();

    let _ = board.led_y.set_state(PinState::from(false));
    let _ = board.led_w.set_state(PinState::from(true));

    let mut last = 0u32;
    loop {
        let count = critical_read_irq_count();
        if count != last {
            last = count;
            let phase = (count & 1) != 0;
            let _ = board.led_y.set_state(PinState::from(phase));
            let _ = board.led_w.set_state(PinState::from(!phase));
        } else if timer::is_timer0_pending() {
            timer::clear_timer0_status();
            timer::set_timer0_irq_capture(TIMER0_PERIOD_TICKS);
            let _ = board.led_y.set_state(PinState::from(true));
            let _ = board.led_w.set_state(PinState::from(true));
        }
    }
}

fn critical_read_irq_count() -> u32 {
    let irq_enabled = interrupt::disable();
    let value = unsafe { *IRQ_COUNT.0.get() };
    interrupt::restore(irq_enabled);
    value
}

#[unsafe(link_section = ".ram_code")]
unsafe extern "C" fn timer0_irq_handler() {
    if interrupt::is_pending(Irq::Timer0) || timer::is_timer0_pending() {
        timer::clear_timer0_status();
        interrupt::clear_irq(Irq::Timer0);
        timer::set_timer0_irq_capture(TIMER0_PERIOD_TICKS);
        unsafe {
            let count = IRQ_COUNT.0.get();
            *count = (*count).wrapping_add(1);
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
