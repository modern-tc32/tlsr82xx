use crate::analog;
use crate::mmio::{reg16, reg32, reg8};
use crate::regs8258::{
    FLD_RF_IRQ_ALL, FLD_RF_IRQ_CMD_DONE, FLD_RF_IRQ_FIRST_TIMEOUT, FLD_RF_IRQ_FSM_TIMEOUT,
    FLD_RF_IRQ_INVALID_PID, FLD_RF_IRQ_RETRY_HIT, FLD_RF_IRQ_RX, FLD_RF_IRQ_RX_CRC_2,
    FLD_RF_IRQ_RX_DR, FLD_RF_IRQ_RX_TIMEOUT, FLD_RF_IRQ_STX_TIMEOUT, FLD_RF_IRQ_TX, FLD_RF_IRQ_TX_DS,
    FLD_RST1_ZB, REG_DMA2_ADDR, REG_DMA2_ADDR_HI, REG_DMA3_ADDR, REG_DMA3_ADDR_HI, REG_DMA_TX_RDY0,
    REG_PLL_RX_FINE_DIV_TUNE, REG_RF_ACCESS_CODE, REG_RF_CHANNEL, REG_RF_CRC, REG_RF_IRQ_MASK,
    REG_RF_IRQ_STATUS, REG_RF_LL_CTRL_0, REG_RF_LL_CTRL_2, REG_RF_LL_CTRL_3, REG_RF_MODE_CONTROL,
    REG_RF_POWER, REG_RF_RSSI, REG_RF_RX_MODE, REG_RF_RX_STATUS, REG_RF_SCHED_TICK, REG_RF_SN,
    REG_RF_TX_SETTLE, REG_RST1,
};

const RF_TRX_MODE: u8 = 0xe0;
const RF_TRX_OFF: u8 = 0x45;
const DEFAULT_TX_SETTLE_US: u16 = 113;
const RF_CMD_BRX: u8 = 0x82;
const RF_CMD_SRX2TX: u8 = 0x85;
const RF_CMD_STX2RX: u8 = 0x87;
const FLD_DMA_CHN_RF_TX: u8 = 1 << 3;
const BLE_ADV_ACCESS_CODE: u32 = 0xd6be898e;
const BLE_ADV_CRC_INIT: u32 = 0x0055_5555;
const REG_RF_BLE_CHANNEL_NUM: usize = 0x0080_040d;
const REG_RF_CHN_SET_L: usize = 0x0080_1244;
const REG_RF_CHN_SET_H: usize = 0x0080_1245;
const REG_RF_CHN_BAND: usize = 0x0080_1228;
const REG_RF_PHY_1220: usize = 0x0080_1220;
const REG_RF_PHY_1273: usize = 0x0080_1273;
const REG_RF_PHY_1236: usize = 0x0080_1236;
const REG_RF_PHY_0401: usize = 0x0080_0401;
const REG_RF_PHY_0402: usize = 0x0080_0402;
const REG_RF_PHY_0420: usize = 0x0080_0420;
const REG_RF_PHY_0460: usize = 0x0080_0460;
const REG_RF_PHY_0464: usize = 0x0080_0464;
const REG_RF_PHY_12D2: usize = 0x0080_12d2;
const REG_RF_PHY_127B: usize = 0x0080_127b;
const REG_RF_PHY_0430: usize = 0x0080_0430;
const REG_RF_PHY_0F06: usize = 0x0080_0f06;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadioMode {
    Ble1M,
    Zigbee250K,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadioError {
    EmptyRxBuffer,
    UnalignedRxBuffer,
    EmptyTxBuffer,
    UnalignedTxBuffer,
    InvalidBleChannel(u8),
    InvalidZigbeeChannel(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadioChannel {
    Ble(u8),
    Zigbee(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BleConfig {
    pub channel: u8,
    pub access_code: u32,
    pub crc: [u8; 3],
    pub power: RadioPower,
}

impl BleConfig {
    #[inline(always)]
    pub const fn advertising(channel: u8) -> Self {
        Self {
            channel,
            access_code: 0x8e89_bed6,
            crc: [0x55, 0x55, 0x55],
            power: RadioPower::PLUS_3P23_DBM,
        }
    }

    #[inline(always)]
    pub const fn data(channel: u8, access_code: u32, crc: [u8; 3]) -> Self {
        Self {
            channel,
            access_code,
            crc,
            power: RadioPower::PLUS_3P23_DBM,
        }
    }

    #[inline(always)]
    pub const fn with_power(self, power: RadioPower) -> Self {
        Self { power, ..self }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZigbeeConfig {
    pub channel: u8,
    pub power: RadioPower,
}

impl ZigbeeConfig {
    #[inline(always)]
    pub const fn new(channel: u8) -> Self {
        Self {
            channel,
            power: RadioPower::PLUS_3P23_DBM,
        }
    }

    #[inline(always)]
    pub const fn with_power(self, power: RadioPower) -> Self {
        Self { power, ..self }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadioConfig {
    Ble(BleConfig),
    Zigbee(ZigbeeConfig),
}

impl RadioConfig {
    #[inline(always)]
    pub const fn mode(self) -> RadioMode {
        match self {
            Self::Ble(_) => RadioMode::Ble1M,
            Self::Zigbee(_) => RadioMode::Zigbee250K,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RadioPower(u8);

impl RadioPower {
    pub const PLUS_10P46_DBM: Self = Self(63);
    pub const PLUS_10P29_DBM: Self = Self(61);
    pub const PLUS_10P01_DBM: Self = Self(58);
    pub const PLUS_9P81_DBM: Self = Self(56);
    pub const PLUS_9P48_DBM: Self = Self(53);
    pub const PLUS_9P24_DBM: Self = Self(51);
    pub const PLUS_8P97_DBM: Self = Self(49);
    pub const PLUS_8P73_DBM: Self = Self(47);
    pub const PLUS_8P44_DBM: Self = Self(45);
    pub const PLUS_8P13_DBM: Self = Self(43);
    pub const PLUS_7P79_DBM: Self = Self(41);
    pub const PLUS_7P41_DBM: Self = Self(39);
    pub const PLUS_7P02_DBM: Self = Self(37);
    pub const PLUS_6P60_DBM: Self = Self(35);
    pub const PLUS_6P14_DBM: Self = Self(33);
    pub const PLUS_5P65_DBM: Self = Self(31);
    pub const PLUS_5P13_DBM: Self = Self(29);
    pub const PLUS_4P57_DBM: Self = Self(27);
    pub const PLUS_3P94_DBM: Self = Self(25);
    pub const PLUS_3P23_DBM: Self = Self(23);
    pub const PLUS_3P01_DBM: Self = Self(0x80 | 63);
    pub const PLUS_2P81_DBM: Self = Self(0x80 | 61);
    pub const PLUS_2P61_DBM: Self = Self(0x80 | 59);
    pub const PLUS_2P39_DBM: Self = Self(0x80 | 57);
    pub const PLUS_1P99_DBM: Self = Self(0x80 | 54);
    pub const PLUS_1P73_DBM: Self = Self(0x80 | 52);
    pub const PLUS_1P45_DBM: Self = Self(0x80 | 50);
    pub const PLUS_1P17_DBM: Self = Self(0x80 | 48);
    pub const PLUS_0P90_DBM: Self = Self(0x80 | 46);
    pub const PLUS_0P58_DBM: Self = Self(0x80 | 44);
    pub const PLUS_0P04_DBM: Self = Self(0x80 | 41);
    pub const MINUS_0P14_DBM: Self = Self(0x80 | 40);
    pub const MINUS_0P97_DBM: Self = Self(0x80 | 36);
    pub const MINUS_1P42_DBM: Self = Self(0x80 | 34);
    pub const MINUS_1P89_DBM: Self = Self(0x80 | 32);
    pub const MINUS_2P48_DBM: Self = Self(0x80 | 30);
    pub const MINUS_3P03_DBM: Self = Self(0x80 | 28);
    pub const MINUS_3P61_DBM: Self = Self(0x80 | 26);
    pub const MINUS_4P26_DBM: Self = Self(0x80 | 24);
    pub const MINUS_5P03_DBM: Self = Self(0x80 | 22);
    pub const MINUS_5P81_DBM: Self = Self(0x80 | 20);
    pub const MINUS_6P67_DBM: Self = Self(0x80 | 18);
    pub const MINUS_7P65_DBM: Self = Self(0x80 | 16);
    pub const MINUS_8P65_DBM: Self = Self(0x80 | 14);
    pub const MINUS_9P89_DBM: Self = Self(0x80 | 12);
    pub const MINUS_11P40_DBM: Self = Self(0x80 | 10);
    pub const MINUS_13P29_DBM: Self = Self(0x80 | 8);
    pub const MINUS_15P88_DBM: Self = Self(0x80 | 6);
    pub const MINUS_19P27_DBM: Self = Self(0x80 | 4);
    pub const MINUS_25P18_DBM: Self = Self(0x80 | 2);
    pub const MINUS_30_DBM: Self = Self(0xff);
    pub const MINUS_50_DBM: Self = Self(0x80);

    #[inline(always)]
    pub const fn raw(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrqFlags(pub u16);

impl IrqFlags {
    pub const RX: Self = Self(FLD_RF_IRQ_RX);
    pub const TX: Self = Self(FLD_RF_IRQ_TX);
    pub const RX_TIMEOUT: Self = Self(FLD_RF_IRQ_RX_TIMEOUT);
    pub const RX_CRC_2: Self = Self(FLD_RF_IRQ_RX_CRC_2);
    pub const CMD_DONE: Self = Self(FLD_RF_IRQ_CMD_DONE);
    pub const FSM_TIMEOUT: Self = Self(FLD_RF_IRQ_FSM_TIMEOUT);
    pub const RETRY_HIT: Self = Self(FLD_RF_IRQ_RETRY_HIT);
    pub const TX_DS: Self = Self(FLD_RF_IRQ_TX_DS);
    pub const RX_DR: Self = Self(FLD_RF_IRQ_RX_DR);
    pub const FIRST_TIMEOUT: Self = Self(FLD_RF_IRQ_FIRST_TIMEOUT);
    pub const INVALID_PID: Self = Self(FLD_RF_IRQ_INVALID_PID);
    pub const STX_TIMEOUT: Self = Self(FLD_RF_IRQ_STX_TIMEOUT);
    pub const ALL: Self = Self(FLD_RF_IRQ_ALL);

    #[inline(always)]
    pub const fn bits(self) -> u16 {
        self.0
    }

    #[inline(always)]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

#[derive(Debug)]
pub struct Radio;

impl Default for Radio {
    fn default() -> Self {
        Self::new()
    }
}

impl Radio {
    #[inline(always)]
    pub const fn new() -> Self {
        Self
    }

    #[inline(always)]
    pub fn reset_baseband(&mut self) {
        unsafe {
            core::ptr::write_volatile(reg8(REG_RST1), FLD_RST1_ZB);
            core::ptr::write_volatile(reg8(REG_RST1), 0);
        }
    }

    #[inline(always)]
    pub fn reset_sn_nesn(&mut self) {
        unsafe {
            core::ptr::write_volatile(reg8(REG_RF_SN), 0x01);
        }
    }

    #[inline(always)]
    pub fn reset_sn(&mut self) {
        unsafe {
            core::ptr::write_volatile(reg8(REG_RF_SN), 0x3f);
            core::ptr::write_volatile(reg8(REG_RF_SN), 0x00);
        }
    }

    #[inline(always)]
    pub fn set_tx_rx_off(&mut self) {
        unsafe {
            core::ptr::write_volatile(reg8(REG_RF_LL_CTRL_3), 0x29);
            core::ptr::write_volatile(reg8(REG_RF_RX_MODE), RF_TRX_MODE);
            core::ptr::write_volatile(reg8(REG_RF_LL_CTRL_0), RF_TRX_OFF);
        }
    }

    #[inline(always)]
    pub fn set_tx_rx_off_auto_mode(&mut self) {
        unsafe {
            core::ptr::write_volatile(reg8(REG_RF_MODE_CONTROL), 0x80);
        }
    }

    #[inline(always)]
    pub fn set_tx_mode(&mut self) {
        unsafe {
            core::ptr::write_volatile(reg8(REG_RF_LL_CTRL_0), RF_TRX_OFF | (1 << 4));
        }
    }

    #[inline(always)]
    pub fn set_rx_mode(&mut self) {
        unsafe {
            core::ptr::write_volatile(reg8(REG_RF_RX_MODE), RF_TRX_MODE | 1);
            core::ptr::write_volatile(reg8(REG_RF_LL_CTRL_0), RF_TRX_OFF | (1 << 5));
        }
    }

    #[inline(always)]
    pub fn set_tx_pipe(&mut self, pipe: u8) {
        unsafe {
            core::ptr::write_volatile(reg8(REG_RF_LL_CTRL_2), 0xf0 | (pipe & 0x0f));
        }
    }

    #[inline(always)]
    pub fn set_tx_settle_us(&mut self, settle_us: u16) {
        let hw = settle_us.saturating_sub(1) & 0x0fff;
        unsafe {
            core::ptr::write_volatile(reg16(REG_RF_TX_SETTLE), hw);
        }
    }

    #[inline(always)]
    pub fn set_rx_buffer(&mut self, buffer: *mut u8) {
        let addr = buffer as usize;
        unsafe {
            core::ptr::write_volatile(reg16(REG_DMA2_ADDR), addr as u16);
            core::ptr::write_volatile(reg8(REG_DMA2_ADDR_HI), ((addr >> 16) as u8) & 0x0f);
        }
    }

    #[inline(always)]
    pub fn configure_rx_buffer(&mut self, buffer: &mut [u8]) -> Result<(), RadioError> {
        if buffer.is_empty() {
            return Err(RadioError::EmptyRxBuffer);
        }
        if (buffer.as_mut_ptr() as usize & 0x3) != 0 {
            return Err(RadioError::UnalignedRxBuffer);
        }
        self.set_rx_buffer(buffer.as_mut_ptr());
        Ok(())
    }

    #[inline(always)]
    pub fn set_tx_buffer(&mut self, buffer: *const u8) {
        let addr = buffer as usize;
        unsafe {
            core::ptr::write_volatile(reg16(REG_DMA3_ADDR), addr as u16);
            core::ptr::write_volatile(reg8(REG_DMA3_ADDR_HI), ((addr >> 16) as u8) & 0x0f);
        }
    }

    #[inline(always)]
    pub fn configure_tx_buffer(&mut self, buffer: &[u8]) -> Result<(), RadioError> {
        if buffer.is_empty() {
            return Err(RadioError::EmptyTxBuffer);
        }
        if (buffer.as_ptr() as usize & 0x3) != 0 {
            return Err(RadioError::UnalignedTxBuffer);
        }
        self.set_tx_buffer(buffer.as_ptr());
        Ok(())
    }

    #[inline(always)]
    pub fn set_power_raw(&mut self, value: u8) {
        unsafe {
            core::ptr::write_volatile(reg8(REG_RF_POWER), value);
        }
    }

    #[inline(always)]
    pub fn set_power(&mut self, power: RadioPower) {
        self.set_power_raw(power.raw());
    }

    #[inline(always)]
    pub fn set_channel_raw(&mut self, channel: u8) {
        unsafe {
            core::ptr::write_volatile(reg8(REG_RF_CHANNEL), channel);
        }
    }

    #[inline(always)]
    pub fn set_access_code(&mut self, access_code: u32) {
        unsafe {
            core::ptr::write_volatile(reg32(REG_RF_ACCESS_CODE), access_code);
        }
    }

    #[inline(always)]
    pub fn set_crc_init(&mut self, crc_init: u32) {
        unsafe {
            core::ptr::write_volatile(reg32(REG_RF_CRC), crc_init & 0x00ff_ffff);
        }
    }

    #[inline(always)]
    pub fn set_ble_advertising_access_code(&mut self) {
        self.set_access_code(BLE_ADV_ACCESS_CODE);
    }

    #[inline(always)]
    pub fn set_ble_access_code(&mut self, access_code: u32) {
        self.set_access_code(access_code.swap_bytes());
    }

    #[inline(always)]
    pub fn set_ble_advertising_crc(&mut self) {
        self.set_crc_init(BLE_ADV_CRC_INIT);
    }

    #[inline(always)]
    pub fn set_ble_crc(&mut self, crc: [u8; 3]) {
        self.set_crc_init((crc[0] as u32) | ((crc[1] as u32) << 8) | ((crc[2] as u32) << 16));
    }

    #[inline(always)]
    fn apply_channel_frequency(&mut self, channel_reg: u8, pll_freq_mhz: u16) {
        self.set_channel_raw(channel_reg);
        analog::write(0x06, 0x00);
        unsafe {
            core::ptr::write_volatile(reg8(REG_RF_LL_CTRL_3), 0x29);
            core::ptr::write_volatile(reg8(REG_RF_RX_MODE), 0x00);
            core::ptr::write_volatile(reg8(REG_RF_LL_CTRL_0), RF_TRX_OFF);
            core::ptr::write_volatile(reg16(REG_PLL_RX_FINE_DIV_TUNE), pll_freq_mhz);
        }
    }

    #[inline(always)]
    pub fn set_channel(&mut self, channel: RadioChannel) -> Result<(), RadioError> {
        match channel {
            RadioChannel::Ble(channel) => self.set_ble_channel(channel),
            RadioChannel::Zigbee(channel) => self.set_zigbee_channel(channel),
        }
    }

    #[inline(always)]
    pub fn set_ble_channel(&mut self, channel: u8) -> Result<(), RadioError> {
        if channel > 39 {
            return Err(RadioError::InvalidBleChannel(channel));
        }
        unsafe {
            core::ptr::write_volatile(reg8(REG_RF_BLE_CHANNEL_NUM), channel);
        }
        let (set, band) = match channel {
            37 => (0x0962u16, 0x18u8),
            38 => (0x097au16, 0x14u8),
            39 => (0x09b0u16, 0x0cu8),
            0..=10 => {
                let set = 0x0960u16 + ((channel as u16) + 2);
                (set, Self::ble_band_bits(set))
            }
            _ => {
                let set = 0x0960u16 + ((channel as u16) + 3);
                (set, Self::ble_band_bits(set))
            }
        };
        self.apply_ble_channel_set(set, band);
        Ok(())
    }

    #[inline(always)]
    fn ble_band_bits(set: u16) -> u8 {
        if set > 0x09be {
            0x04
        } else if set > 0x09a0 {
            0x08
        } else if set > 0x0982 {
            0x0c
        } else if set > 0x0964 {
            0x10
        } else if set > 0x094b {
            0x14
        } else {
            0x18
        }
    }

    #[inline(always)]
    fn apply_ble_channel_set(&mut self, set: u16, band: u8) {
        unsafe {
            core::ptr::write_volatile(reg8(REG_RF_CHN_SET_L), (((set << 2) as u8) | 0x01) as u8);
            let ch_h = reg8(REG_RF_CHN_SET_H);
            let ch_h_new = (core::ptr::read_volatile(ch_h.cast_const()) & !0x3f) | (((set >> 6) as u8) & 0x3f);
            core::ptr::write_volatile(ch_h, ch_h_new);

            let band_reg = reg8(REG_RF_CHN_BAND);
            let band_new = (core::ptr::read_volatile(band_reg.cast_const()) & !0x3c) | (band & 0x3c);
            core::ptr::write_volatile(band_reg, band_new);
        }
    }

    #[inline(always)]
    pub fn set_zigbee_channel(&mut self, channel: u8) -> Result<(), RadioError> {
        if !(11..=26).contains(&channel) {
            return Err(RadioError::InvalidZigbeeChannel(channel));
        }
        let raw = (channel - 10) * 5;
        let pll_freq_mhz = 2405 + ((channel as u16) - 11) * 5;
        self.apply_channel_frequency(raw, pll_freq_mhz);
        Ok(())
    }

    #[inline(always)]
    pub fn set_schedule_tick(&mut self, tick: u32) {
        unsafe {
            core::ptr::write_volatile(reg32(REG_RF_SCHED_TICK), tick);
        }
    }

    #[inline(always)]
    fn enable_scheduled_command(&mut self, command: u8, tick: u32) {
        self.set_schedule_tick(tick);
        unsafe {
            let mode = reg8(REG_RF_LL_CTRL_3);
            let mut value = core::ptr::read_volatile(mode.cast_const());
            value |= 0x04;
            core::ptr::write_volatile(mode, value);
            core::ptr::write_volatile(reg8(REG_RF_MODE_CONTROL), command);
        }
    }

    #[inline(always)]
    pub fn start_brx_at(&mut self, tick: u32) {
        self.enable_scheduled_command(RF_CMD_BRX, tick);
    }

    #[inline(always)]
    pub fn stop_trx(&mut self) {
        unsafe {
            core::ptr::write_volatile(reg8(REG_RF_MODE_CONTROL), 0x80);
        }
    }

    #[inline(always)]
    pub fn start_srx2tx_at(&mut self, tx_packet: &[u8], tick: u32) -> Result<(), RadioError> {
        self.configure_tx_buffer(tx_packet)?;
        self.enable_scheduled_command(RF_CMD_SRX2TX, tick);
        Ok(())
    }

    #[inline(always)]
    pub fn start_srx2tx_now(&mut self, tx_packet: &[u8]) -> Result<(), RadioError> {
        self.configure_tx_buffer(tx_packet)?;
        unsafe {
            let ll_ctrl3 = reg8(REG_RF_LL_CTRL_3);
            let value = core::ptr::read_volatile(ll_ctrl3.cast_const()) & !0x04;
            core::ptr::write_volatile(ll_ctrl3, value);
            core::ptr::write_volatile(reg8(REG_RF_MODE_CONTROL), RF_CMD_SRX2TX);
        }
        Ok(())
    }

    #[inline(always)]
    pub fn tx_packet_now(&mut self, tx_packet: &[u8]) -> Result<(), RadioError> {
        self.configure_tx_buffer(tx_packet)?;
        unsafe {
            core::ptr::write_volatile(reg8(REG_DMA3_ADDR_HI), 0x04);
            let tx_rdy = reg8(REG_DMA_TX_RDY0);
            let value = core::ptr::read_volatile(tx_rdy.cast_const()) | FLD_DMA_CHN_RF_TX;
            core::ptr::write_volatile(tx_rdy, value);
        }
        Ok(())
    }

    #[inline(always)]
    pub fn start_stx2rx_at(&mut self, tx_packet: &[u8], tick: u32) -> Result<(), RadioError> {
        self.configure_tx_buffer(tx_packet)?;
        self.enable_scheduled_command(RF_CMD_STX2RX, tick);
        Ok(())
    }

    #[inline(always)]
    pub fn start_stx2rx_now(&mut self, tx_packet: &[u8]) -> Result<(), RadioError> {
        self.configure_tx_buffer(tx_packet)?;
        unsafe {
            let ll_ctrl3 = reg8(REG_RF_LL_CTRL_3);
            let value = core::ptr::read_volatile(ll_ctrl3.cast_const()) & !0x04;
            core::ptr::write_volatile(ll_ctrl3, value);
            core::ptr::write_volatile(reg8(REG_RF_MODE_CONTROL), RF_CMD_STX2RX);
        }
        Ok(())
    }

    pub fn init_mode(&mut self, mode: RadioMode) -> Result<(), RadioError> {
        self.reset_baseband();
        self.set_tx_rx_off_auto_mode();
        self.set_tx_rx_off();
        self.reset_sn_nesn();
        self.clear_all_irq_status();
        self.clear_irq_mask(IrqFlags::ALL);
        self.set_tx_pipe(0);
        self.set_tx_settle_us(DEFAULT_TX_SETTLE_US);

        match mode {
            RadioMode::Ble1M => self.init_ble_advertising()?,
            RadioMode::Zigbee250K => self.init_zigbee_250k()?,
        }

        Ok(())
    }

    #[inline(always)]
    pub fn apply_config(&mut self, config: RadioConfig) -> Result<(), RadioError> {
        match config {
            RadioConfig::Ble(config) => self.apply_ble_config(config),
            RadioConfig::Zigbee(config) => self.apply_zigbee_config(config),
        }
    }

    #[inline(always)]
    pub fn apply_config_and_start_brx_at(
        &mut self,
        config: RadioConfig,
        tick: u32,
    ) -> Result<(), RadioError> {
        self.apply_config(config)?;
        self.start_brx_at(tick);
        Ok(())
    }

    #[inline(always)]
    pub fn apply_config_and_start_srx2tx_at(
        &mut self,
        config: RadioConfig,
        tx_packet: &[u8],
        tick: u32,
    ) -> Result<(), RadioError> {
        self.apply_config(config)?;
        self.start_srx2tx_at(tx_packet, tick)
    }

    #[inline(always)]
    pub fn apply_config_and_start_stx2rx_at(
        &mut self,
        config: RadioConfig,
        tx_packet: &[u8],
        tick: u32,
    ) -> Result<(), RadioError> {
        self.apply_config(config)?;
        self.start_stx2rx_at(tx_packet, tick)
    }

    #[inline(always)]
    pub fn init_ble_advertising(&mut self) -> Result<(), RadioError> {
        self.apply_ble_phy_init();
        self.apply_ble_config(BleConfig::advertising(37))
    }

    #[inline(always)]
    pub fn init_ble_vendor_1m_phy(&mut self) {
        self.apply_ble_phy_init();
    }

    fn apply_ble_phy_init(&mut self) {
        unsafe {
            // Mirror vendor B85/TLSR8258 BLE 1M bring-up sequence.
            core::ptr::write_volatile(reg8(REG_RF_PHY_12D2), 0x9b);
            core::ptr::write_volatile(reg8(REG_RF_PHY_12D2 + 1), 0x19);
            core::ptr::write_volatile(reg8(REG_RF_PHY_127B), 0x0e);
            core::ptr::write_volatile(reg8(REG_RF_PHY_0430), 0x36);

            core::ptr::write_volatile(reg8(REG_RF_PHY_1220 + 0), 0x16);
            core::ptr::write_volatile(reg8(REG_RF_PHY_1220 + 1), 0x0a);
            core::ptr::write_volatile(reg8(REG_RF_PHY_1220 + 2), 0x20);
            core::ptr::write_volatile(reg8(REG_RF_PHY_1220 + 3), 0x23);
            core::ptr::write_volatile(reg8(REG_RF_PHY_1220 + 0x2a), 0x0e);
            core::ptr::write_volatile(reg8(REG_RF_PHY_1220 + 0x2b), 0x09);
            core::ptr::write_volatile(reg8(REG_RF_PHY_1220 + 0x56), 0x45);
            core::ptr::write_volatile(reg8(REG_RF_PHY_1220 + 0x57), 0x7b);
            core::ptr::write_volatile(reg8(REG_RF_PHY_1220 + 0x59), 0x08);
            core::ptr::write_volatile(reg8(REG_RF_PHY_1273), 0x01);
            core::ptr::write_volatile(reg8(REG_RF_PHY_1236 + 0), 0xb7);
            core::ptr::write_volatile(reg8(REG_RF_PHY_1236 + 1), 0x8e);
            core::ptr::write_volatile(reg8(REG_RF_PHY_1236 + 2), 0xc4);
            core::ptr::write_volatile(reg8(REG_RF_PHY_1236 + 3), 0x71);

            core::ptr::write_volatile(reg8(REG_RF_PHY_0401), 0x01);
            core::ptr::write_volatile(reg8(REG_RF_PHY_0402), 0x46);
            core::ptr::write_volatile(reg8(REG_RF_PHY_0402 + 2), 0xf5);
            core::ptr::write_volatile(reg8(REG_RF_PHY_0402 + 3), 0x04);
            core::ptr::write_volatile(reg8(REG_RF_PHY_0420), 0x1e);
            core::ptr::write_volatile(reg32(REG_RF_PHY_0460), 0x5f4f4434);
            core::ptr::write_volatile(reg16(REG_RF_PHY_0464), 0x766b);

            core::ptr::write_volatile(reg8(REG_RF_PHY_0F06), 0x00);
            core::ptr::write_volatile(reg8(REG_RF_PHY_0F06 + 6), 0x50);
            core::ptr::write_volatile(reg8(REG_RF_PHY_0F06 + 8), 0x00);
            core::ptr::write_volatile(reg8(REG_RF_PHY_0F06 + 10), 0x00);
        }
    }

    #[inline(always)]
    pub fn init_ble_with_access_code_crc(
        &mut self,
        channel: u8,
        access_code: u32,
        crc: [u8; 3],
    ) -> Result<(), RadioError> {
        self.apply_ble_config(BleConfig::data(channel, access_code, crc))
    }

    #[inline(always)]
    pub fn init_zigbee_250k(&mut self) -> Result<(), RadioError> {
        self.apply_zigbee_config(ZigbeeConfig::new(11))
    }

    #[inline(always)]
    pub fn init_zigbee_channel(&mut self, channel: u8) -> Result<(), RadioError> {
        self.apply_zigbee_config(ZigbeeConfig::new(channel))
    }

    #[inline(always)]
    pub fn apply_ble_config(&mut self, config: BleConfig) -> Result<(), RadioError> {
        self.set_power(config.power);
        self.set_ble_channel(config.channel)?;
        self.set_ble_access_code(config.access_code);
        self.set_ble_crc(config.crc);
        Ok(())
    }

    #[inline(always)]
    pub fn apply_zigbee_config(&mut self, config: ZigbeeConfig) -> Result<(), RadioError> {
        self.set_power(config.power);
        self.set_zigbee_channel(config.channel)?;
        Ok(())
    }

    #[inline(always)]
    pub fn irq_mask(&self) -> IrqFlags {
        unsafe { IrqFlags(core::ptr::read_volatile(reg16(REG_RF_IRQ_MASK).cast_const())) }
    }

    #[inline(always)]
    pub fn set_irq_mask(&mut self, flags: IrqFlags) {
        unsafe {
            let reg = reg16(REG_RF_IRQ_MASK);
            let value = core::ptr::read_volatile(reg.cast_const()) | flags.bits();
            core::ptr::write_volatile(reg, value);
        }
    }

    #[inline(always)]
    pub fn clear_irq_mask(&mut self, flags: IrqFlags) {
        unsafe {
            let reg = reg16(REG_RF_IRQ_MASK);
            let value = core::ptr::read_volatile(reg.cast_const()) & !flags.bits();
            core::ptr::write_volatile(reg, value);
        }
    }

    #[inline(always)]
    pub fn irq_status(&self) -> IrqFlags {
        unsafe { IrqFlags(core::ptr::read_volatile(reg16(REG_RF_IRQ_STATUS).cast_const())) }
    }

    #[inline(always)]
    pub fn clear_irq_status(&mut self, flags: IrqFlags) {
        unsafe {
            core::ptr::write_volatile(reg16(REG_RF_IRQ_STATUS), flags.bits());
        }
    }

    #[inline(always)]
    pub fn clear_all_irq_status(&mut self) {
        self.clear_irq_status(IrqFlags::ALL);
    }

    #[inline(always)]
    pub fn tx_finished(&self) -> bool {
        self.irq_status().contains(IrqFlags::TX)
    }

    #[inline(always)]
    pub fn rx_finished(&self) -> bool {
        self.irq_status().contains(IrqFlags::RX)
    }

    #[inline(always)]
    pub fn rssi_dbm_154(&self) -> i8 {
        unsafe { core::ptr::read_volatile(reg8(REG_RF_RSSI).cast_const()) as i8 - 110 }
    }

    #[inline(always)]
    pub fn is_receiving_packet(&self) -> bool {
        unsafe { ((core::ptr::read_volatile(reg8(REG_RF_RX_STATUS).cast_const()) >> 5) & 1) != 0 }
    }
}
