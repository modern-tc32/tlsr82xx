#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::OutputPin;
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::pac;
use tlsr82xx_hal::radio::{BleConfig, IrqFlags, Radio, RadioMode, RadioPower};
use tlsr82xx_hal::timer;

mod platform;

const STATUS_MAGIC: u32 = 0x42_45_41_43; // "BEAC"
const STATUS_VERSION: u32 = 1;

const ADV_INTERVAL_US: u32 = 100_000;
const ADV_TX_TIMEOUT_US: u32 = 4_000;
const ADV_CHANNEL_SETTLE_US: u32 = 180;
const ADV_CHANNELS: [u8; 3] = [37, 38, 39];

const PDU_TYPE_ADV_NONCONN_IND: u8 = 0x02;
const BLE_TX_ADDR_RANDOM_BIT: u8 = 1 << 6;
const BLE_ADV_HEADER0: u8 = PDU_TYPE_ADV_NONCONN_IND | BLE_TX_ADDR_RANDOM_BIT;
const ADV_ADDR_LE: [u8; 6] = [0x58, 0x82, 0xDE, 0xC0, 0xDE, 0xC0];
const ADV_DATA: [u8; 25] = [
    2, 0x01, 0x06, // Flags
    7, 0xFF, 1, 2, 3, 4, 5, 6, // Manufacturer specific data
    13, 0x09, b'T', b'L', b'S', b'R', b'8', b'2', b'5', b'8', b'R', b'U', b'S', b'T', // Complete Local Name
];
const PDU_PAYLOAD_LEN: u8 = (ADV_ADDR_LE.len() + ADV_DATA.len()) as u8;
const PDU_LEN: usize = 2 + (PDU_PAYLOAD_LEN as usize);
const ADV_DMA_LEN: u16 = (PDU_PAYLOAD_LEN as u16) + 2;

const IRQ_TX_SUCCESS_BITS: u16 =
    IrqFlags::TX.bits() | IrqFlags::TX_DS.bits() | IrqFlags::CMD_DONE.bits();
const IRQ_TX_TIMEOUT_BITS: u16 = IrqFlags::STX_TIMEOUT.bits();

const REG_RF_MODE_CONTROL_ADDR: usize = 0x0080_0f00;
const REG_RF_LL_CTRL_0_ADDR: usize = 0x0080_0f02;
const REG_RF_LL_CTRL_3_ADDR: usize = 0x0080_0f16;
const REG_RF_RX_MODE_ADDR: usize = 0x0080_0428;
const REG_RF_IRQ_MASK_ADDR: usize = 0x0080_0f1c;
const REG_RF_IRQ_STATUS_ADDR: usize = 0x0080_0f20;
const REG_RF_BLE_CHNUM_ADDR: usize = 0x0080_040d;
const REG_RF_CHN_SET_L_ADDR: usize = 0x0080_1244;
const REG_RF_CHN_SET_H_ADDR: usize = 0x0080_1245;
const REG_RF_CHN_BAND_ADDR: usize = 0x0080_1228;
const REG_RF_CHN_1244_ADDR: usize = 0x0080_1244;
const REG_DMA3_ADDR_ADDR: usize = 0x0080_0c0c;
const REG_DMA3_SIZE_ADDR: usize = 0x0080_0c0e;
const REG_DMA3_ADDR_HI_ADDR: usize = 0x0080_0c43;
const REG_DMA_TX_RDY0_ADDR: usize = 0x0080_0c24;
const REG_DMA_CHN_EN_ADDR: usize = 0x0080_0c20;
const FLD_DMA_CHN_RF_TX: u8 = 1 << 3;

const PIN_PB2_RAW: u32 = 0x0104;
const PIN_PB3_RAW: u32 = 0x0108;
const GPIO_FUNC_TX_CYC2PA: u8 = 0x20;
const GPIO_FUNC_RX_CYC2LNA: u8 = 0x21;

const PHASE_BOOT: u8 = 1;
const PHASE_INIT_OK: u8 = 2;
const PHASE_PREPARE_EVENT: u8 = 3;
const PHASE_PREPARE_CH: u8 = 4;
const PHASE_TRIGGER_TX: u8 = 5;
const PHASE_WAIT_IRQ: u8 = 6;
const PHASE_TX_OK: u8 = 7;
const PHASE_TX_TIMEOUT: u8 = 8;
const PHASE_TX_ERR: u8 = 9;
const PHASE_EVENT_OK: u8 = 10;
const PHASE_EVENT_FAIL: u8 = 11;

const ERR_NONE: u8 = 0;
const ERR_TIMEOUT: u8 = 4;
const ERR_OTHER_IRQ: u8 = 5;

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
    pub dma3_size: u8,
    pub tx_packet_header0: u8,
    pub dma3_addr: u16,
    pub ble_chn_num: u8,
    pub ble_set_l: u8,
    pub ble_set_h: u8,
    pub ble_band: u8,
    pub rf_1244: u8,
    pub reserved2: [u8; 3],
    pub last_tick: u32,
    pub last_dma_len: u16,
    pub last_pdu_len: u8,
    pub last_adv_data_len: u8,
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
    dma3_size: 0,
    tx_packet_header0: 0,
    dma3_addr: 0,
    ble_chn_num: 0,
    ble_set_l: 0,
    ble_set_h: 0,
    ble_band: 0,
    rf_1244: 0,
    reserved2: [0; 3],
    last_tick: 0,
    last_dma_len: 0,
    last_pdu_len: 0,
    last_adv_data_len: 0,
};

#[unsafe(no_mangle)]
pub static mut BLE_BEACON_STATUS: BleBeaconStatus = INITIAL_STATUS;

#[repr(align(4))]
struct Aligned<const N: usize>([u8; N]);

const TX_BUF_LEN: usize = 64;
static mut TX_PACKET: Aligned<TX_BUF_LEN> = Aligned([0; TX_BUF_LEN]);

#[inline(always)]
fn adv_config(channel: u8) -> BleConfig {
    BleConfig::advertising(channel).with_power(RadioPower::PLUS_10P46_DBM)
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();
    unsafe {
        let boot = BLE_BEACON_STATUS.boot_count.wrapping_add(1);
        BLE_BEACON_STATUS = INITIAL_STATUS;
        BLE_BEACON_STATUS.boot_count = boot;
        BLE_BEACON_STATUS.phase = PHASE_BOOT;
        BLE_BEACON_STATUS.last_dma_len = ADV_DMA_LEN;
        BLE_BEACON_STATUS.last_pdu_len = PDU_LEN as u8;
        BLE_BEACON_STATUS.last_adv_data_len = ADV_DATA.len() as u8;
        BLE_BEACON_STATUS.tx_packet_header0 = BLE_ADV_HEADER0;
    }

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
    let mut radio = Radio::new();
    let _ = board.led_y.set_low();
    let _ = board.led_w.set_low();

    init_rf_path_tb03(&mut radio);
    write_status(|s| {
        s.phase = PHASE_INIT_OK;
        sample_registers(s);
    });

    unsafe { prepare_adv_packet() };
    let mut next_event_at = timer::clock_time().wrapping_sub(ADV_INTERVAL_US);
    loop {
        if !timer::clock_time_exceed_us(next_event_at, ADV_INTERVAL_US) {
            core::hint::spin_loop();
            continue;
        }
        next_event_at = timer::clock_time();
        let _ = board.led_y.set_low();

        write_status(|s| {
            s.loop_count = s.loop_count.wrapping_add(1);
            s.phase = PHASE_PREPARE_EVENT;
            s.last_error = ERR_NONE;
            s.reserved0 = 0;
            s.last_tick = timer::clock_time();
            sample_registers(s);
        });

        let mut event_ok = true;
        for &ch in &ADV_CHANNELS {
            write_status(|s| {
                s.phase = PHASE_PREPARE_CH;
                s.last_channel = ch;
            });

            radio.set_tx_rx_off();
            if radio.apply_ble_config(adv_config(ch)).is_err() {
                event_ok = false;
                write_status(|s| {
                    s.tx_other_irq = s.tx_other_irq.wrapping_add(1);
                    s.last_error = ERR_OTHER_IRQ;
                    s.phase = PHASE_TX_ERR;
                });
                break;
            }

            let settle_start = timer::clock_time();
            while !timer::clock_time_exceed_us(settle_start, ADV_CHANNEL_SETTLE_US) {
                core::hint::spin_loop();
            }

            radio.clear_all_irq_status();
            write_status(|s| {
                s.phase = PHASE_TRIGGER_TX;
                s.tx_attempts = s.tx_attempts.wrapping_add(1);
                sample_registers(s);
            });

            let tx_started = unsafe { start_adv_tx(&mut radio) };
            if !tx_started {
                event_ok = false;
                write_status(|s| {
                    s.tx_other_irq = s.tx_other_irq.wrapping_add(1);
                    s.last_error = ERR_OTHER_IRQ;
                    s.phase = PHASE_TX_ERR;
                });
                break;
            }

            let wait_start = timer::clock_time();
            let mut tx_ok = false;
            let mut tx_timeout = false;
            let mut saw_nonzero_irq = false;
            while !timer::clock_time_exceed_us(wait_start, ADV_TX_TIMEOUT_US) {
                let irq = radio.irq_status().bits();
                write_status(|s| {
                    s.phase = PHASE_WAIT_IRQ;
                    s.last_irq = irq;
                });
                if (irq & IRQ_TX_SUCCESS_BITS) != 0 {
                    radio.clear_irq_status(IrqFlags(IRQ_TX_SUCCESS_BITS));
                    tx_ok = true;
                    break;
                }
                if (irq & IRQ_TX_TIMEOUT_BITS) != 0 {
                    radio.clear_irq_status(IrqFlags(IRQ_TX_TIMEOUT_BITS));
                    tx_timeout = true;
                    break;
                }
                if irq != 0 {
                    saw_nonzero_irq = true;
                    radio.clear_all_irq_status();
                    break;
                }
            }
            if !tx_ok && !tx_timeout && !saw_nonzero_irq {
                tx_timeout = true;
            }

            if tx_ok {
                write_status(|s| {
                    s.tx_ok = s.tx_ok.wrapping_add(1);
                    s.phase = PHASE_TX_OK;
                });
            } else {
                event_ok = false;
                if tx_timeout {
                    write_status(|s| {
                        s.tx_timeout = s.tx_timeout.wrapping_add(1);
                        s.last_error = ERR_TIMEOUT;
                        s.phase = PHASE_TX_TIMEOUT;
                    });
                } else {
                    write_status(|s| {
                        s.tx_other_irq = s.tx_other_irq.wrapping_add(1);
                        s.last_error = ERR_OTHER_IRQ;
                        s.phase = PHASE_TX_ERR;
                    });
                }
                break;
            }
        }

        if event_ok {
            write_status(|s| {
                s.event_ok = s.event_ok.wrapping_add(1);
                s.phase = PHASE_EVENT_OK;
                sample_registers(s);
            });
            pulse_white_20ms(&mut board);
            let _ = board.led_y.set_low();
        } else {
            write_status(|s| {
                s.event_fail = s.event_fail.wrapping_add(1);
                s.phase = PHASE_EVENT_FAIL;
                sample_registers(s);
            });
            let _ = board.led_y.set_high();
        }
    }
}

fn set_rffe_mapping_tb03() {
    unsafe {
        unsafe extern "C" {
            fn gpio_set_func(pin: u32, func: u32);
        }
        gpio_set_func(PIN_PB2_RAW, GPIO_FUNC_RX_CYC2LNA as u32);
        gpio_set_func(PIN_PB3_RAW, GPIO_FUNC_TX_CYC2PA as u32);
    }
}

fn init_rf_path_tb03(radio: &mut Radio) {
    tlsr82xx_hal::clock::init(tlsr82xx_hal::clock::SysClock::Crystal16M);
    set_rffe_mapping_tb03();

    let _ = radio.init_mode(RadioMode::Ble1M);
    let _ = radio.apply_ble_config(adv_config(ADV_CHANNELS[0]));

    let chn = read_u8(REG_DMA_CHN_EN_ADDR);
    write_u8(REG_DMA_CHN_EN_ADDR, chn | FLD_DMA_CHN_RF_TX);

    radio.clear_all_irq_status();
    radio.clear_irq_mask(IrqFlags::ALL);
    radio.set_irq_mask(IrqFlags(IRQ_TX_SUCCESS_BITS | IRQ_TX_TIMEOUT_BITS));
}

unsafe fn start_adv_tx(radio: &mut Radio) -> bool {
    let tx_packet = core::slice::from_raw_parts(core::ptr::addr_of!(TX_PACKET.0).cast::<u8>(), TX_BUF_LEN);
    let tick = timer::clock_time().wrapping_add(10);
    radio.start_srx2tx_at(tx_packet, tick).is_ok()
}

fn write_status(f: impl FnOnce(&mut BleBeaconStatus)) {
    unsafe { f(core::ptr::addr_of_mut!(BLE_BEACON_STATUS).as_mut().unwrap()) }
}

fn sample_registers(s: &mut BleBeaconStatus) {
    s.irq_mask = read_u16(REG_RF_IRQ_MASK_ADDR);
    s.rf_irq_status = read_u16(REG_RF_IRQ_STATUS_ADDR);
    s.rf_mode_ctrl = read_u8(REG_RF_MODE_CONTROL_ADDR);
    s.rf_ll_ctrl0 = read_u8(REG_RF_LL_CTRL_0_ADDR);
    s.rf_ll_ctrl3 = read_u8(REG_RF_LL_CTRL_3_ADDR);
    s.rf_rx_mode = read_u8(REG_RF_RX_MODE_ADDR);
    s.dma_tx_rdy = read_u8(REG_DMA_TX_RDY0_ADDR);
    s.dma_chn_en = read_u8(REG_DMA_CHN_EN_ADDR);
    s.dma3_hi = read_u8(REG_DMA3_ADDR_HI_ADDR);
    s.dma3_size = read_u8(REG_DMA3_SIZE_ADDR);
    s.dma3_addr = read_u16(REG_DMA3_ADDR_ADDR);
    s.ble_chn_num = read_u8(REG_RF_BLE_CHNUM_ADDR);
    s.ble_set_l = read_u8(REG_RF_CHN_SET_L_ADDR);
    s.ble_set_h = read_u8(REG_RF_CHN_SET_H_ADDR);
    s.ble_band = read_u8(REG_RF_CHN_BAND_ADDR);
    s.rf_1244 = read_u8(REG_RF_CHN_1244_ADDR);
}

fn pulse_white_20ms(board: &mut Board) {
    let _ = board.led_w.set_high();
    let start = timer::clock_time();
    while !timer::clock_time_exceed_us(start, 20_000) {
        core::hint::spin_loop();
    }
    let _ = board.led_w.set_low();
}

#[inline(always)]
fn read_u8(addr: usize) -> u8 {
    unsafe { core::ptr::read_volatile(addr as *const u8) }
}

#[inline(always)]
fn read_u16(addr: usize) -> u16 {
    unsafe { core::ptr::read_volatile(addr as *const u16) }
}

#[inline(always)]
fn write_u8(addr: usize, val: u8) {
    unsafe { core::ptr::write_volatile(addr as *mut u8, val) }
}

unsafe fn prepare_adv_packet() {
    let dma_len = ADV_DMA_LEN as u32;
    TX_PACKET.0[0] = (dma_len & 0xff) as u8;
    TX_PACKET.0[1] = ((dma_len >> 8) & 0xff) as u8;
    TX_PACKET.0[2] = 0;
    TX_PACKET.0[3] = 0;
    TX_PACKET.0[4] = BLE_ADV_HEADER0;
    TX_PACKET.0[5] = PDU_PAYLOAD_LEN;
    TX_PACKET.0[6..12].copy_from_slice(&ADV_ADDR_LE);
    let data_start = 12usize;
    let data_end = data_start + ADV_DATA.len();
    TX_PACKET.0[data_start..data_end].copy_from_slice(&ADV_DATA);
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
