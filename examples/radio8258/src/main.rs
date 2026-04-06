#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::{OutputPin, PinState};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::pac;
use tlsr82xx_hal::radio::{BleConfig, IrqFlags, Radio, RadioConfig, ZigbeeConfig};
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
    let ble_config = RadioConfig::Ble(BleConfig::advertising(37));
    let zigbee_config = RadioConfig::Zigbee(ZigbeeConfig::new(11));

    unsafe {
        radio.set_rx_buffer(core::ptr::addr_of_mut!(RX_BUFFER.0).cast::<u8>());
    }
    radio.set_irq_mask(IrqFlags::RX);
    let _ = radio.apply_config_and_start_brx_at(ble_config, timer::clock_time().wrapping_add(32 * 200));
    drive_mode_leds(&mut board, ble_mode);

    loop {
        if timer::clock_time_exceed_us(next_switch, 1_000_000) {
            next_switch = timer::clock_time();
            ble_mode = !ble_mode;

            let _ = if ble_mode {
                radio.apply_config_and_start_brx_at(ble_config, timer::clock_time().wrapping_add(32 * 200))
            } else {
                radio.apply_config_and_start_brx_at(zigbee_config, timer::clock_time().wrapping_add(32 * 200))
            };
            radio.clear_all_irq_status();
            drive_mode_leds(&mut board, ble_mode);
        }
    }
}

fn drive_mode_leds(board: &mut Board, ble_mode: bool) {
    let _ = board.led_y.set_state(PinState::from(ble_mode));
    let _ = board.led_w.set_state(PinState::from(!ble_mode));
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
