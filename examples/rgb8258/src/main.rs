#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::pwm::SetDutyCycle;
use tlsr82xx_boards::tb03f;
use tlsr82xx_hal::gpio::GpioExt;
use tlsr82xx_hal::pac;
use tlsr82xx_hal::pwm::PwmExt;
use tlsr82xx_hal::timer;

mod platform;

const PWM_PERIOD_TICKS: u16 = 48_000;
const BRIGHTNESS_MAX: u16 = 255;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::drv_platform_init();

    let peripherals = unsafe { pac::Peripherals::steal() };
    let mut pins = peripherals.gpio.split();
    tb03f::configure_rgb_pins(&mut pins);
    let pwm = peripherals.pwm.constrain();
    let mut channels = pwm.split();
    channels.pwm0.configure(PWM_PERIOD_TICKS, 0);
    channels.pwm1.configure(PWM_PERIOD_TICKS, 0);
    channels.pwm2.configure(PWM_PERIOD_TICKS, 0);
    channels.pwm0.enable();
    channels.pwm1.enable();
    channels.pwm2.enable();

    let mut phase = 0u16;
    let mut tick = timer::clock_time();

    loop {
        if timer::clock_time_exceed_us(tick, 10_000) {
            tick = timer::clock_time();
            phase = phase.wrapping_add(1) % (BRIGHTNESS_MAX * 6);
            let rgb = wheel(phase);
            let _ = channels.pwm0.set_duty_cycle_fraction(u16::from(rgb.r), 255);
            let _ = channels.pwm1.set_duty_cycle_fraction(u16::from(rgb.g), 255);
            let _ = channels.pwm2.set_duty_cycle_fraction(u16::from(rgb.b), 255);
        }
    }
}

struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

fn wheel(phase: u16) -> Rgb {
    let segment = (phase / BRIGHTNESS_MAX) % 6;
    let offset = (phase % BRIGHTNESS_MAX) as u8;
    let max = BRIGHTNESS_MAX as u8;

    match segment {
        0 => Rgb {
            r: max,
            g: offset,
            b: 0,
        },
        1 => Rgb {
            r: max.saturating_sub(offset),
            g: max,
            b: 0,
        },
        2 => Rgb {
            r: 0,
            g: max,
            b: offset,
        },
        3 => Rgb {
            r: 0,
            g: max.saturating_sub(offset),
            b: max,
        },
        4 => Rgb {
            r: offset,
            g: 0,
            b: max,
        },
        _ => Rgb {
            r: max,
            g: 0,
            b: max.saturating_sub(offset),
        },
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
