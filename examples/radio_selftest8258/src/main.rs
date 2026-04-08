#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::OutputPin;
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::pac;
use tlsr82xx_hal::radio::{IrqFlags, Radio};
use tlsr82xx_hal::timer;

mod platform;

const TX_PERIOD_US: u32 = 1_000_000;
const TX_WAIT_POLL_SPINS: u32 = 200_000;
const IRQ_TX_SUCCESS_BITS: u16 = IrqFlags::TX.bits() | IrqFlags::TX_DS.bits() | IrqFlags::CMD_DONE.bits();
const IRQ_TX_TIMEOUT_BITS: u16 = IrqFlags::STX_TIMEOUT.bits();
const REG_RF_MODE_CONTROL_ADDR: usize = 0x0080_0f00;
const REG_RF_LL_CTRL_0_ADDR: usize = 0x0080_0f02;
const REG_RF_LL_CTRL_3_ADDR: usize = 0x0080_0f16;
const REG_RF_IRQ_MASK_ADDR: usize = 0x0080_0f1c;
const REG_RF_IRQ_STATUS_ADDR: usize = 0x0080_0f20;
const REG_RF_TXRX_CFG1_ADDR: usize = 0x0080_0f03;
const REG_IRQ_MASK_ADDR: usize = 0x0080_0640;
const FLD_IRQ_ZB_RT_EN: u32 = 1 << 13;
const REG_DMA3_ADDR_ADDR: usize = 0x0080_0c0c;
const REG_DMA3_SIZE_ADDR: usize = 0x0080_0c0e;
const REG_DMA3_MODE_ADDR: usize = 0x0080_0c0f;
const REG_DMA3_ADDR_HI_ADDR: usize = 0x0080_0c43;
const REG_DMA_TX_RDY0_ADDR: usize = 0x0080_0c24;
const REG_DMA_CHN_EN_ADDR: usize = 0x0080_0c20;
#[cfg(feature = "selftest-hal")]
const FLD_DMA_CHN_RF_TX: u8 = 1 << 3;
const RF_MODE_ZIGBEE_250K: u32 = 1 << 3;
const RF_STATUS_TX: u32 = 0;

#[cfg(feature = "selftest-hal")]
const MODE_HAL: u8 = 1;
const MODE_VENDOR: u8 = 2;
const PHASE_BOOT: u8 = 1;
const PHASE_INIT_OK: u8 = 2;
const PHASE_PREP_TX: u8 = 3;
const PHASE_TRIGGERED: u8 = 4;
const PHASE_WAIT_TX: u8 = 5;
const PHASE_TX_OK: u8 = 6;
const PHASE_NO_IRQ: u8 = 7;
const PHASE_START_ERR: u8 = 8;

#[cfg(all(feature = "selftest-vendor", feature = "selftest-hal"))]
compile_error!("Enable only one of selftest-vendor or selftest-hal");
#[cfg(not(any(feature = "selftest-vendor", feature = "selftest-hal")))]
compile_error!("Enable one of selftest-vendor or selftest-hal");

const STATUS_MAGIC: u32 = 0x52_46_54_31; // "RFT1"
const STATUS_VERSION: u32 = 1;

const TX_RF_PAYLOAD_LEN: usize = 6;
const TX_DMA_HDR_LEN: usize = 4;
const TX_TOTAL_LEN: usize = TX_DMA_HDR_LEN + 1 + TX_RF_PAYLOAD_LEN;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RadioSelftestStatus {
    pub magic: u32,
    pub version: u32,
    pub loops: u32,
    pub init_ok: u32,
    pub init_err: u32,
    pub tx_attempts: u32,
    pub tx_start_err: u32,
    pub tx_ok: u32,
    pub tx_timeout: u32,
    pub tx_other_irq: u32,
    pub tx_wait_no_irq: u32,
    pub mode: u8,
    pub phase_code: u8,
    pub last_irq: u16,
    pub last_rssi_dbm: i16,
    pub dma3_addr: u16,
    pub dma3_size: u8,
    pub dma3_mode: u8,
    pub dma3_addr_hi: u8,
    pub dma_tx_rdy0: u8,
    pub dma_chn_en: u8,
    pub rf_mode_ctrl: u8,
    pub rf_ll_ctrl0: u8,
    pub rf_ll_ctrl3: u8,
    pub rf_irq_status8: u8,
    pub rf_irq_mask: u16,
    pub rf_irq_status: u16,
}

#[unsafe(no_mangle)]
pub static mut RADIO_SELFTEST_STATUS: RadioSelftestStatus = RadioSelftestStatus {
    magic: STATUS_MAGIC,
    version: STATUS_VERSION,
    loops: 0,
    init_ok: 0,
    init_err: 0,
    tx_attempts: 0,
    tx_start_err: 0,
    tx_ok: 0,
    tx_timeout: 0,
    tx_other_irq: 0,
    tx_wait_no_irq: 0,
    mode: 0,
    phase_code: 0,
    last_irq: 0,
    last_rssi_dbm: 0,
    dma3_addr: 0,
    dma3_size: 0,
    dma3_mode: 0,
    dma3_addr_hi: 0,
    dma_tx_rdy0: 0,
    dma_chn_en: 0,
    rf_mode_ctrl: 0,
    rf_ll_ctrl0: 0,
    rf_ll_ctrl3: 0,
    rf_irq_status8: 0,
    rf_irq_mask: 0,
    rf_irq_status: 0,
};

const INITIAL_STATUS: RadioSelftestStatus = RadioSelftestStatus {
    magic: STATUS_MAGIC,
    version: STATUS_VERSION,
    loops: 0,
    init_ok: 0,
    init_err: 0,
    tx_attempts: 0,
    tx_start_err: 0,
    tx_ok: 0,
    tx_timeout: 0,
    tx_other_irq: 0,
    tx_wait_no_irq: 0,
    mode: 0,
    phase_code: 0,
    last_irq: 0,
    last_rssi_dbm: 0,
    dma3_addr: 0,
    dma3_size: 0,
    dma3_mode: 0,
    dma3_addr_hi: 0,
    dma_tx_rdy0: 0,
    dma_chn_en: 0,
    rf_mode_ctrl: 0,
    rf_ll_ctrl0: 0,
    rf_ll_ctrl3: 0,
    rf_irq_status8: 0,
    rf_irq_mask: 0,
    rf_irq_status: 0,
};

#[repr(align(4))]
struct Aligned<const N: usize>([u8; N]);

static mut RX_BUFFER: Aligned<256> = Aligned([0; 256]);
static mut TX_PACKET: Aligned<TX_TOTAL_LEN> = Aligned([0; TX_TOTAL_LEN]);

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();
    unsafe {
        RADIO_SELFTEST_STATUS = INITIAL_STATUS;
    }

    prepare_tx_packet();

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
    let mut radio = Radio::new();
    write_status(|s| {
        s.mode = active_mode();
        s.phase_code = PHASE_BOOT;
    });

    let _ = board.led_y.set_low();
    let _ = board.led_w.set_low();

    unsafe { radio.set_rx_buffer(core::ptr::addr_of_mut!(RX_BUFFER.0).cast::<u8>()) };

    let init_ok = init_radio_path(&mut radio);
    if init_ok {
        write_status(|s| {
            s.init_ok = s.init_ok.wrapping_add(1);
            s.phase_code = PHASE_INIT_OK;
        });
    } else {
        write_status(|s| s.init_err = s.init_err.wrapping_add(1));
    }

    let mut last_tx_at = timer::clock_time();
    let mut white = false;

    loop {
        if !timer::clock_time_exceed_us(last_tx_at, TX_PERIOD_US) {
            core::hint::spin_loop();
            continue;
        }
        last_tx_at = timer::clock_time();
        write_status(|s| {
            s.loops = s.loops.wrapping_add(1);
            s.phase_code = PHASE_PREP_TX;
            sample_registers(s);
        });

        clear_rf_irq_status();
        prepare_tx_path(&mut radio);
        write_status(|s| s.tx_attempts = s.tx_attempts.wrapping_add(1));

        let start_result = unsafe { trigger_tx(core::ptr::addr_of!(TX_PACKET.0).cast::<u8>()) };

        if start_result.is_err() {
            write_status(|s| {
                s.tx_start_err = s.tx_start_err.wrapping_add(1);
                s.phase_code = PHASE_START_ERR;
                sample_registers(s);
            });
            let _ = board.led_y.set_high();
            let _ = board.led_w.set_low();
            continue;
        }
        write_status(|s| {
            s.phase_code = PHASE_TRIGGERED;
            sample_registers(s);
        });

        let mut wait_spins = 0u32;
        let mut tx_done = false;
        while wait_spins < TX_WAIT_POLL_SPINS {
            wait_spins = wait_spins.wrapping_add(1);
            let irq8 = read_u8(REG_RF_IRQ_STATUS_ADDR);
            let irq = u16::from(irq8);
            write_status(|s| {
                s.last_irq = irq;
                s.phase_code = PHASE_WAIT_TX;
            });

            if (irq & IRQ_TX_SUCCESS_BITS) != 0 {
                radio.clear_irq_status(IrqFlags(IRQ_TX_SUCCESS_BITS));
                write_status(|s| s.tx_ok = s.tx_ok.wrapping_add(1));
                tx_done = true;
                break;
            }

            if (irq & IRQ_TX_TIMEOUT_BITS) != 0 {
                radio.clear_irq_status(IrqFlags(IRQ_TX_TIMEOUT_BITS));
                write_status(|s| s.tx_timeout = s.tx_timeout.wrapping_add(1));
                break;
            }

            if irq != 0 {
                radio.clear_all_irq_status();
                write_status(|s| s.tx_other_irq = s.tx_other_irq.wrapping_add(1));
                break;
            }
        }

        if tx_done {
            write_status(|s| s.phase_code = PHASE_TX_OK);
            white = !white;
            let _ = if white {
                board.led_w.set_high()
            } else {
                board.led_w.set_low()
            };
            let _ = board.led_y.set_low();
        } else {
            write_status(|s| {
                s.tx_wait_no_irq = s.tx_wait_no_irq.wrapping_add(1);
                s.phase_code = PHASE_NO_IRQ;
            });
            let _ = board.led_y.set_high();
            let _ = board.led_w.set_low();
        }

        write_status(|s| {
            s.last_rssi_dbm = i16::from(radio.rssi_dbm_154());
            sample_registers(s);
        });
    }
}

fn write_status(f: impl FnOnce(&mut RadioSelftestStatus)) {
    unsafe {
        f(core::ptr::addr_of_mut!(RADIO_SELFTEST_STATUS).as_mut().unwrap());
    }
}

fn prepare_tx_packet() {
    unsafe {
        // 8258 SDK Zigbee path (rf_tx_pkt): DMA header is a plain byte length.
        TX_PACKET.0[0] = (TX_RF_PAYLOAD_LEN as u8) + 1;
        TX_PACKET.0[1] = 0;
        TX_PACKET.0[2] = 0;
        TX_PACKET.0[3] = 0;
        TX_PACKET.0[4] = (TX_RF_PAYLOAD_LEN as u8) + 2;
        TX_PACKET.0[5] = 0x61;
        TX_PACKET.0[6] = 0x88;
        TX_PACKET.0[7] = 0x52;
        TX_PACKET.0[8] = 0xAD;
        TX_PACKET.0[9] = 0x01;
        TX_PACKET.0[10] = 0x00;
    }
}

#[inline(always)]
fn read_u8(addr: usize) -> u8 {
    unsafe { core::ptr::read_volatile(addr as *const u8) }
}

#[inline(always)]
fn read_u16(addr: usize) -> u16 {
    unsafe { core::ptr::read_volatile(addr as *const u16) }
}

#[cfg(feature = "selftest-hal")]
#[inline(always)]
unsafe fn trigger_manual_tx(packet_ptr: *const u8) -> Result<(), ()> {
    if (packet_ptr as usize & 0x3) != 0 {
        return Err(());
    }

    let addr = packet_ptr as usize;
    core::ptr::write_volatile(REG_DMA3_ADDR_ADDR as *mut u16, addr as u16);
    core::ptr::write_volatile(REG_DMA3_ADDR_HI_ADDR as *mut u8, ((addr >> 16) as u8) & 0x0f);

    let tx_rdy = core::ptr::read_volatile(REG_DMA_TX_RDY0_ADDR as *const u8);
    core::ptr::write_volatile(REG_DMA_TX_RDY0_ADDR as *mut u8, tx_rdy | FLD_DMA_CHN_RF_TX);
    Ok(())
}

#[inline(always)]
fn sample_registers(s: &mut RadioSelftestStatus) {
    s.dma3_addr = read_u16(REG_DMA3_ADDR_ADDR);
    s.dma3_size = read_u8(REG_DMA3_SIZE_ADDR);
    s.dma3_mode = read_u8(REG_DMA3_MODE_ADDR);
    s.dma3_addr_hi = read_u8(REG_DMA3_ADDR_HI_ADDR);
    s.dma_tx_rdy0 = read_u8(REG_DMA_TX_RDY0_ADDR);
    s.dma_chn_en = read_u8(REG_DMA_CHN_EN_ADDR);
    s.rf_mode_ctrl = read_u8(REG_RF_MODE_CONTROL_ADDR);
    s.rf_ll_ctrl0 = read_u8(REG_RF_LL_CTRL_0_ADDR);
    s.rf_ll_ctrl3 = read_u8(REG_RF_LL_CTRL_3_ADDR);
    s.rf_irq_status8 = read_u8(REG_RF_IRQ_STATUS_ADDR);
    s.rf_irq_mask = read_u16(REG_RF_IRQ_MASK_ADDR);
    s.rf_irq_status = read_u16(REG_RF_IRQ_STATUS_ADDR);
}

#[inline(always)]
fn clear_rf_irq_status() {
    unsafe {
        core::ptr::write_volatile(REG_RF_IRQ_STATUS_ADDR as *mut u16, 0xffff);
    }
}

#[inline(always)]
fn active_mode() -> u8 {
    #[cfg(feature = "selftest-vendor")]
    {
        MODE_VENDOR
    }
    #[cfg(feature = "selftest-hal")]
    {
        MODE_HAL
    }
}

#[inline(always)]
fn init_radio_path(radio: &mut Radio) -> bool {
    #[cfg(feature = "selftest-vendor")]
    unsafe {
        let _ = radio;
        vendor_rf_init();
        true
    }
    #[cfg(feature = "selftest-hal")]
    {
        use tlsr82xx_hal::radio::RadioMode;
        if radio.init_mode(RadioMode::Zigbee250K).is_err() {
            return false;
        }
        radio.set_irq_mask(IrqFlags(
            IrqFlags::TX.bits()
                | IrqFlags::TX_DS.bits()
                | IrqFlags::CMD_DONE.bits()
                | IrqFlags::STX_TIMEOUT.bits(),
        ));
        true
    }
}

#[inline(always)]
fn prepare_tx_path(radio: &mut Radio) {
    #[cfg(feature = "selftest-vendor")]
    unsafe {
        let _ = radio;
        vendor_rf_prepare_tx();
    }
    #[cfg(feature = "selftest-hal")]
    {
        unsafe {
            core::ptr::write_volatile(REG_RF_LL_CTRL_3_ADDR as *mut u8, 0x19);
        }
        radio.set_tx_mode();
    }
}

#[inline(always)]
unsafe fn trigger_tx(packet_ptr: *const u8) -> Result<(), ()> {
    #[cfg(feature = "selftest-vendor")]
    {
        vendor_rf_tx_pkt(packet_ptr as *mut u8);
        Ok(())
    }
    #[cfg(feature = "selftest-hal")]
    {
        trigger_manual_tx(packet_ptr)
    }
}

#[cfg(feature = "selftest-vendor")]
unsafe fn vendor_rf_init() {
    unsafe extern "C" {
        fn rf_drv_init(mode: u32);
        fn rf_set_channel(chn: i8, set: u16);
        fn rf_set_power_level(level: u8);
        fn rf_rx_cfg(size: i32, pingpong: u8);
        fn rf_rx_buffer_set(addr: *mut u8, size: i32, pingpong: u8);
    }

    rf_drv_init(RF_MODE_ZIGBEE_250K);
    rf_set_channel(11, 0);
    rf_set_power_level(23);
    rf_rx_cfg(128, 0);
    rf_rx_buffer_set(core::ptr::addr_of_mut!(RX_BUFFER.0).cast::<u8>(), 128, 0);
    core::ptr::write_volatile(0x0080_0f15 as *mut u8, 0xf0);
    core::ptr::write_volatile(0x0080_0f04 as *mut u16, 113);
    let cfg1 = core::ptr::read_volatile(REG_RF_TXRX_CFG1_ADDR as *const u8);
    core::ptr::write_volatile(REG_RF_TXRX_CFG1_ADDR as *mut u8, cfg1 & !(1 << 2));
    let irq_msk = core::ptr::read_volatile(REG_IRQ_MASK_ADDR as *const u32);
    core::ptr::write_volatile(REG_IRQ_MASK_ADDR as *mut u32, irq_msk | FLD_IRQ_ZB_RT_EN);
    core::ptr::write_volatile(
        REG_RF_IRQ_MASK_ADDR as *mut u16,
        IrqFlags::RX.bits() | IrqFlags::TX.bits(),
    );
}

#[cfg(feature = "selftest-vendor")]
unsafe fn vendor_rf_prepare_tx() {
    unsafe extern "C" {
        fn rf_trx_state_set(status: u32, channel: i8) -> i32;
    }
    let _ = rf_trx_state_set(RF_STATUS_TX, 11);
}

#[cfg(feature = "selftest-vendor")]
unsafe fn vendor_rf_tx_pkt(packet_ptr: *mut u8) {
    unsafe extern "C" {
        fn rf_tx_pkt(addr: *mut u8);
    }

    // vendor rf_tx_pkt/rf_start_stx update only DMA3 low 16 bits, so keep DMA3 high nibble in sync
    let addr = packet_ptr as usize;
    core::ptr::write_volatile(REG_DMA3_ADDR_HI_ADDR as *mut u8, ((addr >> 16) as u8) & 0x0f);
    rf_tx_pkt(packet_ptr);
}

#[cfg(feature = "selftest-vendor")]
#[unsafe(no_mangle)]
pub extern "C" fn sleep_us(us: u32) {
    sleep_us_compat(us);
}

#[inline(always)]
fn sleep_us_compat(delay_us: u32) {
    let start = timer::clock_time();
    while !timer::clock_time_exceed_us(start, delay_us) {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
