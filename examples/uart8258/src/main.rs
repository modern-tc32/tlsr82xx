#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_io::Write as _;
use tlsr82xx_boards::tb03f;
use tlsr82xx_hal::gpio::GpioExt;
use tlsr82xx_hal::pac;
use tlsr82xx_hal::timer;
use tlsr82xx_hal::uart::{Config, UartExt};

mod platform;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::drv_platform_init();

    let peripherals = unsafe { pac::Peripherals::steal() };
    let mut pins = peripherals.gpio.split();
    tb03f::configure_uart_pins(&mut pins);
    let mut uart = peripherals.uart.constrain();
    uart.configure(Config::new(115_200));

    let _ = embedded_io::Write::write_fmt(&mut uart, format_args!("tlsr82xx uart8258 ready\r\n"));
    let _ = uart.write_all(b"embedded-io write_all ready\r\n");
    let mut counter = 0u32;
    let mut tick = timer::clock_time();

    loop {
        if timer::clock_time_exceed_us(tick, 500_000) {
            tick = timer::clock_time();
            let _ = embedded_io::Write::write_fmt(&mut uart, format_args!("tick {}\r\n", counter));
            let _ = embedded_io::Write::flush(&mut uart);
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
