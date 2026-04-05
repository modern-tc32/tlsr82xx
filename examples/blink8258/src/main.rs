#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::OutputPin;
use tlsr82xx_hal::gpio::{DriveStrength, GpioExt, Level};
use tlsr82xx_hal::pac;

mod platform;
mod time;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    unsafe {
        let _ = platform::drv_platform_init();
    }

    let peripherals = unsafe { pac::Peripherals::steal() };
    let pins = peripherals.gpio.split();
    let mut led_y = pins.pb4.into_output_with_state(Level::High);
    let mut led_w = pins.pb5.into_output_with_state(Level::Low);
    let mut tick = time::clock_time();
    let mut led_y_on = true;

    led_y.set_drive_strength(DriveStrength::Strong);
    led_w.set_drive_strength(DriveStrength::Strong);

    loop {
        if time::clock_time_exceed(tick, 500_000) {
            tick = time::clock_time();
            led_y_on = !led_y_on;
            if led_y_on {
                drive_pin(&mut led_y, true);
                drive_pin(&mut led_w, false);
            } else {
                drive_pin(&mut led_y, false);
                drive_pin(&mut led_w, true);
            }
        }
    }
}

fn drive_pin<P: OutputPin>(pin: &mut P, high: bool) {
    let _ = if high { pin.set_high() } else { pin.set_low() };
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
