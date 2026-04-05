#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

use tlsr82xx_boards::tb03f;
use tlsr82xx_hal::pac;
use tlsr82xx_hal::timer;
use tlsr82xx_hal::uart::{Config, UartExt};

mod platform;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::drv_platform_init();
    tb03f::configure_uart_pins();

    let peripherals = unsafe { pac::Peripherals::steal() };
    let mut uart = peripherals.uart.constrain();
    uart.configure(Config::new(115_200, 48_000_000));

    let _ = writeln!(uart, "tlsr82xx uart8258 ready");
    let mut counter = 0u32;
    let mut tick = timer::clock_time();

    loop {
        if timer::clock_time_exceed_us(tick, 500_000) {
            tick = timer::clock_time();
            let _ = writeln!(uart, "tick {}", counter);
            uart.flush();
            counter = counter.wrapping_add(1);
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
