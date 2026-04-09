#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::OutputPin;
use tlsr82xx_ble::{BeaconAdvertiser, BeaconConfig, BeaconFailureReason};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::gpio::{self, PinFunction};
use tlsr82xx_hal::pac;
use tlsr82xx_hal::radio::{Radio, RadioPower};
use tlsr82xx_hal::timer;

mod platform;

const STATUS_MAGIC: u32 = 0x42_45_41_43; // "BEAC"
const STATUS_VERSION: u32 = 2;

const ADV_ADDR_LE: [u8; 6] = [0x58, 0x82, 0xDE, 0xC0, 0xDE, 0xC0];
const ADV_DATA: [u8; 25] = [
    2, 0x01, 0x06, // Flags
    7, 0xFF, 1, 2, 3, 4, 5, 6, // Manufacturer specific data
    13, 0x09, b'T', b'L', b'S', b'R', b'8', b'2', b'5', b'8', b'R', b'U', b'S', b'T', // Complete Local Name
];

const PHASE_BOOT: u8 = 1;
const PHASE_INIT_OK: u8 = 2;
const PHASE_WAIT_EVENT: u8 = 3;
const PHASE_EVENT_OK: u8 = 10;
const PHASE_EVENT_FAIL: u8 = 11;

const ERR_NONE: u8 = 0;
const ERR_TIMEOUT: u8 = 4;
const ERR_OTHER_IRQ: u8 = 5;
const ERR_CONFIG: u8 = 6;
const ERR_START_TX: u8 = 7;

const PIN_PB2_RAW: u16 = 0x0104;
const PIN_PB3_RAW: u16 = 0x0108;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BleBeaconStatus {
    pub magic: u32,
    pub version: u32,
    pub boot_count: u32,
    pub loop_count: u32,
    pub phase: u8,
    pub last_error: u8,
    pub last_channel: u8,
    pub reserved0: u8,
    pub event_ok: u32,
    pub event_fail: u32,
    pub tx_attempts: u32,
    pub tx_ok: u32,
    pub tx_timeout: u32,
    pub tx_other_irq: u32,
    pub last_irq: u16,
    pub reserved1: u16,
    pub irq_mask: u16,
    pub rf_irq_status: u16,
    pub rf_mode_ctrl: u8,
    pub rf_ll_ctrl0: u8,
    pub rf_ll_ctrl3: u8,
    pub rf_rx_mode: u8,
    pub dma_tx_rdy: u8,
    pub dma_chn_en: u8,
    pub dma3_hi: u8,
    pub reserved2: u8,
    pub dma3_addr: u16,
    pub ble_chn_num: u8,
    pub ble_set_l: u8,
    pub ble_set_h: u8,
    pub ble_band: u8,
    pub last_tick: u32,
}

const INITIAL_STATUS: BleBeaconStatus = BleBeaconStatus {
    magic: STATUS_MAGIC,
    version: STATUS_VERSION,
    boot_count: 0,
    loop_count: 0,
    phase: 0,
    last_error: 0,
    last_channel: 0,
    reserved0: 0,
    event_ok: 0,
    event_fail: 0,
    tx_attempts: 0,
    tx_ok: 0,
    tx_timeout: 0,
    tx_other_irq: 0,
    last_irq: 0,
    reserved1: 0,
    irq_mask: 0,
    rf_irq_status: 0,
    rf_mode_ctrl: 0,
    rf_ll_ctrl0: 0,
    rf_ll_ctrl3: 0,
    rf_rx_mode: 0,
    dma_tx_rdy: 0,
    dma_chn_en: 0,
    dma3_hi: 0,
    reserved2: 0,
    dma3_addr: 0,
    ble_chn_num: 0,
    ble_set_l: 0,
    ble_set_h: 0,
    ble_band: 0,
    last_tick: 0,
};

#[unsafe(no_mangle)]
pub static mut BLE_BEACON_STATUS: BleBeaconStatus = INITIAL_STATUS;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();
    unsafe {
        let boot = BLE_BEACON_STATUS.boot_count.wrapping_add(1);
        BLE_BEACON_STATUS = INITIAL_STATUS;
        BLE_BEACON_STATUS.boot_count = boot;
        BLE_BEACON_STATUS.phase = PHASE_BOOT;
    }

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
    let _ = board.led_y.set_low();
    let _ = board.led_w.set_low();

    tlsr82xx_hal::clock::init(tlsr82xx_hal::clock::SysClock::Crystal16M);
    set_rffe_mapping_tb03();

    let radio = Radio::new();
    let config = BeaconConfig {
        power: RadioPower::PLUS_10P46_DBM,
        ..BeaconConfig::default()
    };
    let mut beacon = match BeaconAdvertiser::new(radio, config, ADV_ADDR_LE, &ADV_DATA) {
        Ok(v) => v,
        Err(_) => loop {
            let _ = board.led_y.set_high();
            core::hint::spin_loop();
        },
    };
    if beacon.init().is_err() {
        loop {
            let _ = board.led_y.set_high();
            core::hint::spin_loop();
        }
    }

    write_status(|s| {
        s.phase = PHASE_INIT_OK;
        sample_radio(s, beacon.radio());
    });

    loop {
        if !beacon.should_run_event() {
            core::hint::spin_loop();
            continue;
        }
        write_status(|s| {
            s.phase = PHASE_WAIT_EVENT;
            s.last_error = ERR_NONE;
            s.last_tick = timer::clock_time();
            sample_radio(s, beacon.radio());
        });

        let Some(result) = beacon.run_event_if_due() else {
            continue;
        };

        write_status(|s| {
            s.loop_count = s.loop_count.wrapping_add(1);
            s.last_channel = result.failed_channel;
            s.last_irq = result.last_irq;
            s.tx_attempts = s.tx_attempts.wrapping_add(result.tx_attempts as u32);
            s.tx_ok = s.tx_ok.wrapping_add(result.tx_ok as u32);
            s.tx_timeout = s.tx_timeout.wrapping_add(result.tx_timeout as u32);
            s.tx_other_irq = s.tx_other_irq.wrapping_add(result.tx_other_irq as u32);
            s.last_tick = timer::clock_time();
            sample_radio(s, beacon.radio());
        });

        if result.ok {
            write_status(|s| {
                s.event_ok = s.event_ok.wrapping_add(1);
                s.phase = PHASE_EVENT_OK;
            });
            pulse_white_20ms(&mut board);
            let _ = board.led_y.set_low();
        } else {
            write_status(|s| {
                s.event_fail = s.event_fail.wrapping_add(1);
                s.phase = PHASE_EVENT_FAIL;
                s.last_error = match result.failure_reason {
                    Some(BeaconFailureReason::Timeout) => ERR_TIMEOUT,
                    Some(BeaconFailureReason::UnexpectedIrq) => ERR_OTHER_IRQ,
                    Some(BeaconFailureReason::Config) => ERR_CONFIG,
                    Some(BeaconFailureReason::StartTx) => ERR_START_TX,
                    None => ERR_OTHER_IRQ,
                };
            });
            let _ = board.led_y.set_high();
        }
    }
}

fn set_rffe_mapping_tb03() {
    gpio::set_function_for_raw_pin(PIN_PB2_RAW, PinFunction::RxCyc2Lna);
    gpio::set_function_for_raw_pin(PIN_PB3_RAW, PinFunction::TxCyc2Pa);
}

fn write_status(f: impl FnOnce(&mut BleBeaconStatus)) {
    unsafe { f(core::ptr::addr_of_mut!(BLE_BEACON_STATUS).as_mut().unwrap()) }
}

fn sample_radio(s: &mut BleBeaconStatus, radio: &Radio) {
    let snapshot = radio.debug_snapshot();
    s.irq_mask = snapshot.irq_mask;
    s.rf_irq_status = snapshot.irq_status;
    s.rf_mode_ctrl = snapshot.mode_ctrl;
    s.rf_ll_ctrl0 = snapshot.ll_ctrl0;
    s.rf_ll_ctrl3 = snapshot.ll_ctrl3;
    s.rf_rx_mode = snapshot.rx_mode;
    s.dma_tx_rdy = snapshot.dma_tx_rdy;
    s.dma_chn_en = snapshot.dma_chn_en;
    s.dma3_hi = snapshot.dma3_addr_hi;
    s.dma3_addr = snapshot.dma3_addr;
    s.ble_chn_num = snapshot.ble_chn_num;
    s.ble_set_l = snapshot.ble_set_l;
    s.ble_set_h = snapshot.ble_set_h;
    s.ble_band = snapshot.ble_band;
}

fn pulse_white_20ms(board: &mut Board) {
    let _ = board.led_w.set_high();
    let start = timer::clock_time();
    while !timer::clock_time_exceed_us(start, 20_000) {
        core::hint::spin_loop();
    }
    let _ = board.led_w.set_low();
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
