#![no_std]
#![no_main]

use core::panic::PanicInfo;

use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::pac;
use tlsr82xx_hal::radio::{IrqFlags, Radio, RadioMode};
use tlsr82xx_hal::timer;

mod platform;

#[repr(align(4))]
struct Aligned<const N: usize>([u8; N]);

static mut RX_BUFFER: Aligned<256> = Aligned([0; 256]);

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
    let mut radio = Radio::new();
    let mut ble_mode = true;
    let mut next_switch = timer::clock_time();

    unsafe {
        radio.set_rx_buffer(core::ptr::addr_of_mut!(RX_BUFFER.0).cast::<u8>());
    }
    radio.set_irq_mask(IrqFlags::RX);
    let _ = radio.init_mode(RadioMode::Ble1M);
    radio.start_brx_at(timer::clock_time().wrapping_add(32 * 200));
    drive_mode_leds(&mut board, ble_mode);

    loop {
        if timer::clock_time_exceed_us(next_switch, 1_000_000) {
            next_switch = timer::clock_time();
            ble_mode = !ble_mode;

            let _ = if ble_mode {
                radio.init_mode(RadioMode::Ble1M)
            } else {
                radio.init_mode(RadioMode::Zigbee250K)
            };
            radio.clear_all_irq_status();
            radio.start_brx_at(timer::clock_time().wrapping_add(32 * 200));
            drive_mode_leds(&mut board, ble_mode);
        }
    }
}

fn drive_mode_leds(board: &mut Board, ble_mode: bool) {
    if ble_mode {
        let _ = board.led_y.set_high();
        let _ = board.led_w.set_low();
    } else {
        let _ = board.led_y.set_low();
        let _ = board.led_w.set_high();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
