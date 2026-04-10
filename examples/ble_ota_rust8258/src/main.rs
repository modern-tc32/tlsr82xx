#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::OutputPin;
use tlsr82xx_ble::{OtaBootConfig, OtaLinkState, OtaLinkTransition, OtaPeripheral};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::pac;
use tlsr82xx_hal::radio::{Radio, RadioPower};
use tlsr82xx_hal::timer;

mod platform;

const STATUS_MAGIC: u32 = 0x4F_54_41_52; // "OTAR"
const STATUS_VERSION: u32 = 9;

const ADV_ADDR_LE: [u8; 6] = [0x01, 0x00, 0x58, 0x82, 0xDE, 0xC0]; // C0:DE:82:58:00:01
const PHASE_BOOT: u8 = 1;
const PHASE_BOARD_OK: u8 = 2;
const PHASE_CLOCK_OK: u8 = 3;
const PHASE_OTA_NEW_OK: u8 = 4;
const PHASE_INIT_OK: u8 = 5;
const PHASE_WAIT_EVENT: u8 = 6;
const PHASE_EVENT_OK: u8 = 10;
const PHASE_EVENT_FAIL: u8 = 11;
const PHASE_CONNECTED: u8 = 20;
const PHASE_DISCONNECTED: u8 = 21;

const ERR_NONE: u8 = 0;
const ERR_TIMEOUT: u8 = 4;
const ERR_OTHER_IRQ: u8 = 5;
const ERR_CONFIG: u8 = 6;
const ERR_OTA_NEW: u8 = 7;
const ERR_OTA_INIT: u8 = 8;

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
    pub conn_events: u32,
    pub disconn_events: u32,
    pub link_activity_events: u32,
    pub link_state: u8,
    pub last_rx_pdu_type: u8,
    pub last_rx_pdu_len: u8,
    pub last_rx_target_match: u8,
    pub reserved3: u8,
    pub last_rx_init_addr0: u8,
    pub last_rx_init_addr1: u8,
    pub last_conn_aa: u32,
    pub last_conn_interval: u16,
    pub last_conn_timeout: u16,
    pub last_conn_hop: u8,
    pub conn_listen_armed: u8,
    pub conn_data_channel: u8,
    pub reserved4: u8,
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
    pub conn_data_rx_count: u32,
    pub last_conn_data_llid: u8,
    pub last_conn_data_len: u8,
    pub reserved5: [u8; 2],
    pub conn_ll_ctrl_rx_count: u32,
    pub last_conn_ll_ctrl_opcode: u8,
    pub reserved8: [u8; 3],
    pub conn_att_rx_count: u32,
    pub last_conn_att_opcode: u8,
    pub reserved6: [u8; 3],
    pub conn_att_rsp_count: u32,
    pub conn_att_tx_attempt_count: u32,
    pub conn_att_tx_ok_count: u32,
    pub last_conn_att_rsp_opcode: u8,
    pub reserved7: [u8; 3],
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
    conn_events: 0,
    disconn_events: 0,
    link_activity_events: 0,
    link_state: 0,
    last_rx_pdu_type: 0,
    last_rx_pdu_len: 0,
    last_rx_target_match: 0,
    reserved3: 0,
    last_rx_init_addr0: 0,
    last_rx_init_addr1: 0,
    last_conn_aa: 0,
    last_conn_interval: 0,
    last_conn_timeout: 0,
    last_conn_hop: 0,
    conn_listen_armed: 0,
    conn_data_channel: 0,
    reserved4: 0,
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
    conn_data_rx_count: 0,
    last_conn_data_llid: 0,
    last_conn_data_len: 0,
    reserved5: [0; 2],
    conn_ll_ctrl_rx_count: 0,
    last_conn_ll_ctrl_opcode: 0,
    reserved8: [0; 3],
    conn_att_rx_count: 0,
    last_conn_att_opcode: 0,
    reserved6: [0; 3],
    conn_att_rsp_count: 0,
    conn_att_tx_attempt_count: 0,
    conn_att_tx_ok_count: 0,
    last_conn_att_rsp_opcode: 0,
    reserved7: [0; 3],
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
    write_status(|s| s.phase = PHASE_BOARD_OK);

    tlsr82xx_hal::clock::init(tlsr82xx_hal::clock::SysClock::Crystal16M);
    write_status(|s| s.phase = PHASE_CLOCK_OK);
    let radio = Radio::new();
    let mut ota = match OtaPeripheral::new(
        radio,
        OtaBootConfig {
            adv_addr_le: ADV_ADDR_LE,
            adv_interval_us: 100_000,
            adv_power: RadioPower::PLUS_10P46_DBM,
            connection_hold_us: 2_000_000,
            ..OtaBootConfig::default()
        },
    ) {
        Ok(v) => v,
        Err(_) => loop {
            write_status(|s| {
                s.last_error = ERR_OTA_NEW;
                s.phase = PHASE_EVENT_FAIL;
            });
            let _ = board.led_y.set_high();
            core::hint::spin_loop();
        },
    };
    write_status(|s| s.phase = PHASE_OTA_NEW_OK);
    if ota.init().is_err() {
        loop {
            write_status(|s| {
                s.last_error = ERR_OTA_INIT;
                s.phase = PHASE_EVENT_FAIL;
                sample_radio(s, ota.radio());
            });
            let _ = board.led_y.set_high();
            core::hint::spin_loop();
        }
    }

    write_status(|s| {
        s.phase = PHASE_INIT_OK;
        sample_radio(s, ota.radio());
    });

    loop {
        write_status(|s| {
            s.phase = PHASE_WAIT_EVENT;
            s.last_error = ERR_NONE;
            s.last_tick = timer::clock_time();
            sample_radio(s, ota.radio());
        });

        let run = ota.run_once();

        write_status(|s| match run.link_state {
            OtaLinkState::Advertising => s.link_state = 0,
            OtaLinkState::Connected => s.link_state = 1,
        });
        if let Some(rx) = run.rx_meta {
            write_status(|s| {
                s.last_rx_pdu_type = rx.pdu_type;
                s.last_rx_pdu_len = rx.pdu_len;
                s.last_rx_target_match = if rx.target_match { 1 } else { 0 };
                s.last_rx_init_addr0 = rx.initiator_addr_le[0];
                s.last_rx_init_addr1 = rx.initiator_addr_le[1];
            });
        }
        if let Some(conn) = run.connect_ind {
            write_status(|s| {
                s.last_conn_aa = conn.access_addr;
                s.last_conn_interval = conn.interval;
                s.last_conn_timeout = conn.timeout;
                s.last_conn_hop = conn.hop;
            });
        }
        if run.conn_listen_armed {
            write_status(|s| {
                s.conn_listen_armed = 1;
                s.conn_data_channel = run.conn_data_channel;
            });
        }
        if run.conn_data_rx {
            write_status(|s| {
                s.conn_data_rx_count = s.conn_data_rx_count.wrapping_add(1);
                s.last_conn_data_llid = run.conn_data_llid;
                s.last_conn_data_len = run.conn_data_len;
            });
        }
        if run.conn_ll_ctrl_rx {
            write_status(|s| {
                s.conn_ll_ctrl_rx_count = s.conn_ll_ctrl_rx_count.wrapping_add(1);
                s.last_conn_ll_ctrl_opcode = run.conn_ll_ctrl_opcode;
            });
        }
        if run.conn_att_rx {
            write_status(|s| {
                s.conn_att_rx_count = s.conn_att_rx_count.wrapping_add(1);
                s.last_conn_att_opcode = run.conn_att_opcode;
            });
        }
        if run.conn_att_rsp_built {
            write_status(|s| {
                s.conn_att_rsp_count = s.conn_att_rsp_count.wrapping_add(1);
                s.last_conn_att_rsp_opcode = run.conn_att_rsp_opcode;
            });
        }
        if run.conn_att_tx_attempted {
            write_status(|s| {
                s.conn_att_tx_attempt_count = s.conn_att_tx_attempt_count.wrapping_add(1);
                if run.conn_att_tx_ok {
                    s.conn_att_tx_ok_count = s.conn_att_tx_ok_count.wrapping_add(1);
                }
            });
        }

        write_status(|s| match run.transition {
            OtaLinkTransition::Connected => {
                s.conn_events = s.conn_events.wrapping_add(1);
                s.phase = PHASE_CONNECTED;
            }
            OtaLinkTransition::Disconnected => {
                s.disconn_events = s.disconn_events.wrapping_add(1);
                s.phase = PHASE_DISCONNECTED;
            }
            OtaLinkTransition::Activity => {
                s.link_activity_events = s.link_activity_events.wrapping_add(1);
            }
            OtaLinkTransition::None => {}
        });

        let Some(result) = run.adv_event else {
            core::hint::spin_loop();
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
            sample_radio(s, ota.radio());
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
                s.last_error = if result.tx_timeout != 0 {
                    ERR_TIMEOUT
                } else if result.tx_other_irq != 0 {
                    ERR_OTHER_IRQ
                } else {
                    ERR_CONFIG
                };
            });
            let _ = board.led_y.set_high();
        }
    }
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
