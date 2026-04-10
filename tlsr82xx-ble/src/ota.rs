use tlsr82xx_hal::flash::{Flash, PAGE_SIZE, SECTOR_SIZE};
use tlsr82xx_hal::radio::{BleConfig, IrqFlags, Radio, RadioPower};
use tlsr82xx_hal::timer;

use crate::beacon::{BeaconAdvertiser, BeaconConfig, BeaconError, BeaconEventResult, BeaconRxMeta};

const CMD_OTA_VERSION: u16 = 0xFF00;
const CMD_OTA_START: u16 = 0xFF01;
const CMD_OTA_END: u16 = 0xFF02;
const CMD_OTA_START_EXT: u16 = 0xFF03;
const CMD_OTA_FW_VERSION_REQ: u16 = 0xFF04;
const CMD_OTA_FW_VERSION_RSP: u16 = 0xFF05;
const CMD_OTA_RESULT: u16 = 0xFF06;
const CMD_OTA_SCHEDULE_FW_SIZE: u16 = 0xFF09;

const OTA_SUCCESS: u8 = 0;
const OTA_PACKET_INVALID: u8 = 2;
const OTA_WRITE_FLASH_ERR: u8 = 4;
const OTA_DATA_INCOMPLETE: u8 = 5;
const OTA_FLOW_ERR: u8 = 6;
const OTA_FW_SIZE_ERR: u8 = 11;
const ATT_MTU_DEFAULT: u16 = 23;
const ATT_OP_ERROR_RSP: u8 = 0x01;
const ATT_OP_EXCHANGE_MTU_REQ: u8 = 0x02;
const ATT_OP_EXCHANGE_MTU_RSP: u8 = 0x03;
const ATT_OP_FIND_INFO_REQ: u8 = 0x04;
const ATT_OP_FIND_INFO_RSP: u8 = 0x05;
const ATT_OP_READ_BY_TYPE_REQ: u8 = 0x08;
const ATT_OP_READ_BY_TYPE_RSP: u8 = 0x09;
const ATT_OP_READ_REQ: u8 = 0x0A;
const ATT_OP_READ_RSP: u8 = 0x0B;
const ATT_OP_READ_BY_GROUP_TYPE_REQ: u8 = 0x10;
const ATT_OP_READ_BY_GROUP_TYPE_RSP: u8 = 0x11;
const ATT_OP_WRITE_REQ: u8 = 0x12;
const ATT_OP_WRITE_RSP: u8 = 0x13;
const ATT_OP_HANDLE_VALUE_NOTIF: u8 = 0x1B;
const ATT_OP_WRITE_CMD: u8 = 0x52;
const ATT_ERR_INVALID_HANDLE: u8 = 0x01;
const ATT_ERR_WRITE_NOT_PERMITTED: u8 = 0x03;
const ATT_ERR_INVALID_PDU: u8 = 0x04;
const ATT_ERR_ATTRIBUTE_NOT_FOUND: u8 = 0x0A;

pub const TELINK_OTA_SERVICE_UUID: [u8; 16] = [
    0x12, 0x19, 0x0d, 0x0c, 0x0b, 0x0a, 0x09, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00,
];
pub const TELINK_SPP_DATA_OTA_UUID: [u8; 16] = [
    0x12, 0x2B, 0x0d, 0x0c, 0x0b, 0x0a, 0x09, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00,
];

const GATT_UUID_PRIMARY_SERVICE: u16 = 0x2800;
const GATT_UUID_CHARACTER: u16 = 0x2803;
const GATT_UUID_DEVICE_NAME: u16 = 0x2A00;
const GATT_UUID_SERVICE_CHANGE: u16 = 0x2A05;
const GATT_UUID_CLIENT_CHAR_CFG: u16 = 0x2902;
const GATT_UUID_CHAR_USER_DESC: u16 = 0x2901;
const CHAR_PROP_READ: u8 = 0x02;
const CHAR_PROP_INDICATE: u8 = 0x20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum OtaAttributeHandle {
    GapPrimaryService = 0x0001,
    GapDeviceNameChar = 0x0002,
    GapDeviceNameValue = 0x0003,
    GattPrimaryService = 0x0004,
    GattServiceChangedChar = 0x0005,
    GattServiceChangedValue = 0x0006,
    GattServiceChangedCcc = 0x0007,
    OtaPrimaryService = 0x0008,
    OtaDataChar = 0x0009,
    OtaDataValue = 0x000A,
    OtaDataCcc = 0x000B,
    OtaDataUserDesc = 0x000C,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OtaAttribute {
    pub handle: u16,
    pub uuid16: Option<u16>,
    pub uuid128: Option<[u8; 16]>,
}

pub const fn default_ota_attributes() -> [OtaAttribute; 12] {
    [
        OtaAttribute {
            handle: OtaAttributeHandle::GapPrimaryService as u16,
            uuid16: Some(GATT_UUID_PRIMARY_SERVICE),
            uuid128: None,
        },
        OtaAttribute {
            handle: OtaAttributeHandle::GapDeviceNameChar as u16,
            uuid16: Some(GATT_UUID_CHARACTER),
            uuid128: None,
        },
        OtaAttribute {
            handle: OtaAttributeHandle::GapDeviceNameValue as u16,
            uuid16: Some(GATT_UUID_DEVICE_NAME),
            uuid128: None,
        },
        OtaAttribute {
            handle: OtaAttributeHandle::GattPrimaryService as u16,
            uuid16: Some(GATT_UUID_PRIMARY_SERVICE),
            uuid128: None,
        },
        OtaAttribute {
            handle: OtaAttributeHandle::GattServiceChangedChar as u16,
            uuid16: Some(GATT_UUID_CHARACTER),
            uuid128: None,
        },
        OtaAttribute {
            handle: OtaAttributeHandle::GattServiceChangedValue as u16,
            uuid16: Some(GATT_UUID_SERVICE_CHANGE),
            uuid128: None,
        },
        OtaAttribute {
            handle: OtaAttributeHandle::GattServiceChangedCcc as u16,
            uuid16: Some(GATT_UUID_CLIENT_CHAR_CFG),
            uuid128: None,
        },
        OtaAttribute {
            handle: OtaAttributeHandle::OtaPrimaryService as u16,
            uuid16: Some(GATT_UUID_PRIMARY_SERVICE),
            uuid128: Some(TELINK_OTA_SERVICE_UUID),
        },
        OtaAttribute {
            handle: OtaAttributeHandle::OtaDataChar as u16,
            uuid16: Some(GATT_UUID_CHARACTER),
            uuid128: None,
        },
        OtaAttribute {
            handle: OtaAttributeHandle::OtaDataValue as u16,
            uuid16: None,
            uuid128: Some(TELINK_SPP_DATA_OTA_UUID),
        },
        OtaAttribute {
            handle: OtaAttributeHandle::OtaDataCcc as u16,
            uuid16: Some(GATT_UUID_CLIENT_CHAR_CFG),
            uuid128: None,
        },
        OtaAttribute {
            handle: OtaAttributeHandle::OtaDataUserDesc as u16,
            uuid16: Some(GATT_UUID_CHAR_USER_DESC),
            uuid128: None,
        },
    ]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct OtaBootConfig {
    pub app_slot_base: u32,
    pub app_slot_size: u32,
    pub metadata_addr: u32,
    pub firmware_version: u16,
    pub adv_addr_le: [u8; 6],
    pub adv_interval_us: u32,
    pub adv_power: RadioPower,
    pub connection_hold_us: u32,
}

impl Default for OtaBootConfig {
    fn default() -> Self {
        Self {
            app_slot_base: 0x0000_8000,
            app_slot_size: 0x0007_7000,
            metadata_addr: 0x0007_F000,
            firmware_version: 1,
            adv_addr_le: [0x58, 0x82, 0xDE, 0xC0, 0x4F, 0x54],
            adv_interval_us: 50_000,
            adv_power: RadioPower::PLUS_10P46_DBM,
            connection_hold_us: 2_000_000,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct OtaStatus {
    pub started: bool,
    pub ccc_enabled: bool,
    pub pdu_len: u8,
    pub expected_index: u16,
    pub highest_written_index: u16,
    pub bytes_written: u32,
    pub ota_result: u8,
    pub fw_version: u16,
}

impl OtaStatus {
    const fn new(fw_version: u16) -> Self {
        Self {
            started: false,
            ccc_enabled: false,
            pdu_len: 16,
            expected_index: 0,
            highest_written_index: 0,
            bytes_written: 0,
            ota_result: OTA_SUCCESS,
            fw_version,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OtaCommandError {
    InvalidLength,
    InvalidFlow,
    InvalidIndex,
    InvalidPduLength,
    InvalidSize,
    FlashWrite,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OtaProcessResult {
    pub schedule_notify: Option<[u8; 6]>,
    pub result_notify: Option<[u8; 4]>,
}

impl OtaProcessResult {
    const fn none() -> Self {
        Self {
            schedule_notify: None,
            result_notify: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OtaEventResult {
    Idle,
    Advertising(BeaconEventResult),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OtaLinkState {
    Advertising,
    Connected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OtaLinkTransition {
    None,
    Connected,
    Disconnected,
    Activity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OtaRunResult {
    pub adv_event: Option<BeaconEventResult>,
    pub rx_meta: Option<BeaconRxMeta>,
    pub connect_ind: Option<OtaConnectIndSummary>,
    pub conn_listen_armed: bool,
    pub conn_data_channel: u8,
    pub conn_data_rx: bool,
    pub conn_data_llid: u8,
    pub conn_data_len: u8,
    pub conn_ll_ctrl_rx: bool,
    pub conn_ll_ctrl_opcode: u8,
    pub conn_att_rx: bool,
    pub conn_att_opcode: u8,
    pub conn_att_rsp_built: bool,
    pub conn_att_rsp_opcode: u8,
    pub conn_att_tx_attempted: bool,
    pub conn_att_tx_ok: bool,
    pub link_state: OtaLinkState,
    pub transition: OtaLinkTransition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OtaConnectIndSummary {
    pub init_addr_le: [u8; 6],
    pub access_addr: u32,
    pub crc_init: [u8; 3],
    pub channel_map: [u8; 5],
    pub interval: u16,
    pub latency: u16,
    pub timeout: u16,
    pub hop: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct OtaImageMetadata {
    magic: u32,
    state: u32,
    app_size: u32,
    reserved: u32,
}

const OTA_META_MAGIC: u32 = 0x4F54_4131;
const OTA_META_STATE_READY: u32 = 0x5245_4144;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OtaConnSchedule {
    access_addr: u32,
    crc_init: [u8; 3],
    channel_map: [u8; 5],
    hop: u8,
    interval_us: u32,
    event_counter: u16,
    last_arm_tick: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OtaAttResponseReport {
    built: bool,
    rsp_opcode: u8,
    tx_attempted: bool,
    tx_ok: bool,
}

pub struct OtaPeripheral {
    advertiser: BeaconAdvertiser,
    flash: Flash,
    config: OtaBootConfig,
    status: OtaStatus,
    erased_until: u32,
    mtu: u16,
    pending_notify: [u8; 8],
    pending_notify_len: u8,
    link_state: OtaLinkState,
    last_link_activity_tick: u32,
    conn_data_channel: u8,
    conn_schedule: Option<OtaConnSchedule>,
}

impl OtaPeripheral {
    pub fn new(radio: Radio, config: OtaBootConfig) -> Result<Self, BeaconError> {
        let (adv_data, adv_len) = build_adv_data();
        let beacon = BeaconAdvertiser::new(
            radio,
            BeaconConfig {
                interval_us: config.adv_interval_us,
                power: config.adv_power,
                connectable: true,
                ..BeaconConfig::default()
            },
            config.adv_addr_le,
            &adv_data[..adv_len],
        )?;

        Ok(Self {
            advertiser: beacon,
            flash: Flash::new(),
            config,
            status: OtaStatus::new(config.firmware_version),
            erased_until: config.app_slot_base,
            mtu: ATT_MTU_DEFAULT,
            pending_notify: [0; 8],
            pending_notify_len: 0,
            link_state: OtaLinkState::Advertising,
            last_link_activity_tick: timer::clock_time(),
            conn_data_channel: 0,
            conn_schedule: None,
        })
    }

    pub fn init(&mut self) -> Result<(), BeaconError> {
        self.advertiser.init()
    }

    #[inline(always)]
    pub fn status(&self) -> OtaStatus {
        self.status
    }

    #[inline(always)]
    pub fn adv_addr_le(&self) -> [u8; 6] {
        self.config.adv_addr_le
    }

    #[inline(always)]
    pub fn adv_power(&self) -> RadioPower {
        self.config.adv_power
    }

    pub fn run_event_if_due(&mut self) -> OtaEventResult {
        match self.advertiser.run_event_if_due() {
            Some(r) => OtaEventResult::Advertising(r),
            None => OtaEventResult::Idle,
        }
    }

    pub fn run_once(&mut self) -> OtaRunResult {
        let adv_event = self.advertiser.run_event_if_due();
        let rx_meta = self.advertiser.take_last_rx_meta();
        let mut rx_packet = [0u8; 80];
        let rx_packet_len = self.advertiser.take_last_rx_packet(&mut rx_packet);
        let conn_data = parse_connection_data_report(
            self.link_state == OtaLinkState::Connected,
            self.conn_schedule.is_some(),
            rx_meta,
            if rx_packet_len != 0 {
                Some(&rx_packet[..rx_packet_len])
            } else {
                None
            },
        );
        if conn_data.rx {
            self.last_link_activity_tick = timer::clock_time();
        }
        let connect_ind = if rx_packet_len != 0 {
            parse_connect_ind_summary(rx_meta, &rx_packet[..rx_packet_len])
        } else {
            None
        };
        let mut att_rsp = OtaAttResponseReport {
            built: false,
            rsp_opcode: 0,
            tx_attempted: false,
            tx_ok: false,
        };
        if rx_packet_len != 0 {
            att_rsp = self.handle_att_on_connection_data(&rx_packet[..rx_packet_len], conn_data);
        }
        let mut conn_listen_armed = false;
        if let Some(summary) = connect_ind {
            conn_listen_armed = self.arm_connection_data_listen(summary);
        }
        if self.link_state == OtaLinkState::Connected && self.maybe_rearm_connection_data_listen() {
            conn_listen_armed = true;
        }
        let transition = self.update_link_state(adv_event, rx_meta, connect_ind.is_some());
        OtaRunResult {
            adv_event,
            rx_meta,
            connect_ind,
            conn_listen_armed,
            conn_data_channel: self.conn_data_channel,
            conn_data_rx: conn_data.rx,
            conn_data_llid: conn_data.llid,
            conn_data_len: conn_data.len,
            conn_ll_ctrl_rx: conn_data.ll_ctrl_opcode != 0,
            conn_ll_ctrl_opcode: conn_data.ll_ctrl_opcode,
            conn_att_rx: conn_data.att_opcode != 0,
            conn_att_opcode: conn_data.att_opcode,
            conn_att_rsp_built: att_rsp.built,
            conn_att_rsp_opcode: att_rsp.rsp_opcode,
            conn_att_tx_attempted: att_rsp.tx_attempted,
            conn_att_tx_ok: att_rsp.tx_ok,
            link_state: self.link_state,
            transition,
        }
    }

    #[inline(always)]
    pub fn link_state(&self) -> OtaLinkState {
        self.link_state
    }

    #[inline(always)]
    pub fn radio(&self) -> &Radio {
        self.advertiser.radio()
    }

    #[inline(always)]
    pub fn radio_mut(&mut self) -> &mut Radio {
        self.advertiser.radio_mut()
    }

    fn arm_connection_data_listen(&mut self, summary: OtaConnectIndSummary) -> bool {
        // Minimal post-connect data-channel listen path.
        let channel = pick_data_channel(summary.channel_map, summary.hop, 0);
        let radio = self.advertiser.radio_mut();
        if radio
            .apply_ble_config(BleConfig::data(channel, summary.access_addr, summary.crc_init))
            .is_err()
        {
            return false;
        }
        let at = timer::clock_time().wrapping_add(32 * 80);
        radio.start_brx_at(at);
        self.conn_data_channel = channel;
        self.conn_schedule = Some(OtaConnSchedule {
            access_addr: summary.access_addr,
            crc_init: summary.crc_init,
            channel_map: summary.channel_map,
            hop: summary.hop,
            interval_us: conn_interval_us(summary.interval),
            event_counter: 1,
            last_arm_tick: timer::clock_time(),
        });
        true
    }

    fn maybe_rearm_connection_data_listen(&mut self) -> bool {
        let Some(mut sched) = self.conn_schedule else {
            return false;
        };
        if sched.interval_us == 0 || !timer::clock_time_exceed_us(sched.last_arm_tick, sched.interval_us) {
            return false;
        }
        let channel = pick_data_channel(sched.channel_map, sched.hop, sched.event_counter);
        let radio = self.advertiser.radio_mut();
        if radio
            .apply_ble_config(BleConfig::data(channel, sched.access_addr, sched.crc_init))
            .is_err()
        {
            return false;
        }
        let at = timer::clock_time().wrapping_add(32 * 80);
        radio.start_brx_at(at);
        sched.event_counter = sched.event_counter.wrapping_add(1);
        sched.last_arm_tick = timer::clock_time();
        self.conn_data_channel = channel;
        self.conn_schedule = Some(sched);
        true
    }

    fn handle_att_on_connection_data(
        &mut self,
        packet: &[u8],
        report: OtaConnDataReport,
    ) -> OtaAttResponseReport {
        if !report.rx || report.llid != 0x02 || report.att_opcode == 0 {
            return OtaAttResponseReport {
                built: false,
                rsp_opcode: 0,
                tx_attempted: false,
                tx_ok: false,
            };
        }
        if packet.len() < 11 {
            return OtaAttResponseReport {
                built: false,
                rsp_opcode: 0,
                tx_attempted: false,
                tx_ok: false,
            };
        }
        let l2cap_len = u16::from_le_bytes([packet[6], packet[7]]) as usize;
        let cid = u16::from_le_bytes([packet[8], packet[9]]);
        if cid != 0x0004 {
            return OtaAttResponseReport {
                built: false,
                rsp_opcode: 0,
                tx_attempted: false,
                tx_ok: false,
            };
        }
        if packet.len() < 10 + l2cap_len || l2cap_len == 0 {
            return OtaAttResponseReport {
                built: false,
                rsp_opcode: 0,
                tx_attempted: false,
                tx_ok: false,
            };
        }
        let att_req = &packet[10..(10 + l2cap_len)];
        let mut att_rsp = [0u8; 64];
        let mut rsp_len = self.handle_att_pdu(att_req, &mut att_rsp);
        if rsp_len == 0 {
            rsp_len = self.take_pending_notify(&mut att_rsp);
        }
        if rsp_len == 0 {
            return OtaAttResponseReport {
                built: false,
                rsp_opcode: 0,
                tx_attempted: false,
                tx_ok: false,
            };
        }

        let mut tx_packet = [0u8; 80];
        let tx_len = build_ll_att_tx_packet(&att_rsp[..rsp_len], &mut tx_packet);
        if tx_len == 0 {
            return OtaAttResponseReport {
                built: true,
                rsp_opcode: att_rsp[0],
                tx_attempted: false,
                tx_ok: false,
            };
        }
        let tx_ok = self.try_send_connection_data_now(&tx_packet[..tx_len]);
        OtaAttResponseReport {
            built: true,
            rsp_opcode: att_rsp[0],
            tx_attempted: true,
            tx_ok,
        }
    }

    fn try_send_connection_data_now(&mut self, tx_packet: &[u8]) -> bool {
        let Some(sched) = self.conn_schedule else {
            return false;
        };
        let radio = self.advertiser.radio_mut();
        if radio
            .apply_ble_config(BleConfig::data(
                self.conn_data_channel,
                sched.access_addr,
                sched.crc_init,
            ))
            .is_err()
        {
            return false;
        }
        radio.start_stx2rx_now(tx_packet).is_ok()
    }

    fn update_link_state(
        &mut self,
        adv_event: Option<BeaconEventResult>,
        rx_meta: Option<BeaconRxMeta>,
        connect_ind_seen: bool,
    ) -> OtaLinkTransition {
        const CONNECTABLE_RX_BITS: u16 = IrqFlags::RX.bits() | IrqFlags::RX_DR.bits();
        const BLE_PDU_SCAN_REQ: u8 = 0x03;
        let now = timer::clock_time();

        if let Some(meta) = rx_meta {
            if meta.pdu_type == BLE_PDU_SCAN_REQ && meta.target_match {
                // Scanner presence on air: useful liveness signal while still advertising.
                return OtaLinkTransition::Activity;
            }
        }

        if let Some(event) = adv_event {
            if (event.last_irq & CONNECTABLE_RX_BITS) != 0 && connect_ind_seen {
                self.last_link_activity_tick = now;
                let transition = if self.link_state == OtaLinkState::Advertising {
                    OtaLinkTransition::Connected
                } else {
                    OtaLinkTransition::Activity
                };
                self.link_state = OtaLinkState::Connected;
                return transition;
            }
        }

        if self.link_state == OtaLinkState::Connected
            && timer::clock_time_exceed_us(self.last_link_activity_tick, self.config.connection_hold_us)
        {
            self.link_state = OtaLinkState::Advertising;
            self.conn_schedule = None;
            return OtaLinkTransition::Disconnected;
        }

        OtaLinkTransition::None
    }

    pub fn process_att_write(
        &mut self,
        handle: u16,
        data: &[u8],
    ) -> Result<OtaProcessResult, OtaCommandError> {
        if handle == OtaAttributeHandle::OtaDataCcc as u16 {
            if data.len() < 2 {
                return Err(OtaCommandError::InvalidLength);
            }
            self.status.ccc_enabled = (u16::from_le_bytes([data[0], data[1]]) & 0x0001) != 0;
            return Ok(OtaProcessResult::none());
        }

        if handle != OtaAttributeHandle::OtaDataValue as u16 {
            return Ok(OtaProcessResult::none());
        }

        if data.len() >= 2 {
            let cmd = u16::from_le_bytes([data[0], data[1]]);
            match cmd {
                CMD_OTA_VERSION | CMD_OTA_FW_VERSION_REQ => {
                    let rsp = build_version_rsp(self.status.fw_version);
                    return Ok(OtaProcessResult {
                        schedule_notify: None,
                        result_notify: Some(rsp),
                    });
                }
                CMD_OTA_START => {
                    self.start_session(16)?;
                    return Ok(OtaProcessResult::none());
                }
                CMD_OTA_START_EXT => {
                    if data.len() < 4 {
                        return Err(OtaCommandError::InvalidLength);
                    }
                    self.start_session(data[2])?;
                    return Ok(OtaProcessResult::none());
                }
                CMD_OTA_END => {
                    if data.len() < 6 {
                        return Err(OtaCommandError::InvalidLength);
                    }
                    let idx = u16::from_le_bytes([data[2], data[3]]);
                    let idx_xor = u16::from_le_bytes([data[4], data[5]]);
                    if (idx ^ idx_xor) != 0xFFFF {
                        self.status.ota_result = OTA_PACKET_INVALID;
                        return Err(OtaCommandError::InvalidIndex);
                    }
                    if !self.status.started {
                        self.status.ota_result = OTA_FLOW_ERR;
                        return Err(OtaCommandError::InvalidFlow);
                    }
                    if self.status.bytes_written == 0 {
                        self.status.ota_result = OTA_DATA_INCOMPLETE;
                        return Err(OtaCommandError::InvalidFlow);
                    }
                    if idx != self.status.highest_written_index {
                        self.status.ota_result = OTA_DATA_INCOMPLETE;
                        return Err(OtaCommandError::InvalidIndex);
                    }
                    self.commit_image()?;
                    self.status.started = false;
                    self.status.ota_result = OTA_SUCCESS;
                    let result = build_result_notify(OTA_SUCCESS);
                    return Ok(OtaProcessResult {
                        schedule_notify: None,
                        result_notify: Some(result),
                    });
                }
                _ => {}
            }
        }

        self.handle_data_chunk(data)
    }

    pub fn handle_att_pdu(&mut self, pdu: &[u8], out: &mut [u8]) -> usize {
        if pdu.is_empty() || out.is_empty() {
            return 0;
        }
        match pdu[0] {
            ATT_OP_EXCHANGE_MTU_REQ => self.handle_att_exchange_mtu_req(pdu, out),
            ATT_OP_READ_BY_GROUP_TYPE_REQ => self.handle_att_read_by_group_type_req(pdu, out),
            ATT_OP_READ_BY_TYPE_REQ => self.handle_att_read_by_type_req(pdu, out),
            ATT_OP_FIND_INFO_REQ => self.handle_att_find_info_req(pdu, out),
            ATT_OP_READ_REQ => self.handle_att_read_req(pdu, out),
            ATT_OP_WRITE_REQ => self.handle_att_write_req(pdu, out),
            ATT_OP_WRITE_CMD => {
                self.handle_att_write_cmd(pdu);
                0
            }
            _ => self.write_att_error(out, pdu[0], 0, ATT_ERR_INVALID_PDU),
        }
    }

    pub fn take_pending_notify(&mut self, out: &mut [u8]) -> usize {
        let payload_len = self.pending_notify_len as usize;
        if payload_len == 0 || out.len() < payload_len + 3 {
            return 0;
        }
        out[0] = ATT_OP_HANDLE_VALUE_NOTIF;
        out[1..3].copy_from_slice(&(OtaAttributeHandle::OtaDataValue as u16).to_le_bytes());
        out[3..(3 + payload_len)].copy_from_slice(&self.pending_notify[..payload_len]);
        self.pending_notify_len = 0;
        payload_len + 3
    }

    fn queue_notify(&mut self, payload: &[u8]) {
        let len = if payload.len() > self.pending_notify.len() {
            self.pending_notify.len()
        } else {
            payload.len()
        };
        self.pending_notify[..len].copy_from_slice(&payload[..len]);
        self.pending_notify_len = len as u8;
    }

    fn handle_att_exchange_mtu_req(&mut self, pdu: &[u8], out: &mut [u8]) -> usize {
        if pdu.len() < 3 || out.len() < 3 {
            return self.write_att_error(out, ATT_OP_EXCHANGE_MTU_REQ, 0, ATT_ERR_INVALID_PDU);
        }
        let client_mtu = u16::from_le_bytes([pdu[1], pdu[2]]);
        self.mtu = if client_mtu < ATT_MTU_DEFAULT {
            ATT_MTU_DEFAULT
        } else {
            client_mtu
        };
        out[0] = ATT_OP_EXCHANGE_MTU_RSP;
        out[1..3].copy_from_slice(&self.mtu.to_le_bytes());
        3
    }

    fn handle_att_read_by_group_type_req(&mut self, pdu: &[u8], out: &mut [u8]) -> usize {
        if pdu.len() < 7 {
            return self.write_att_error(out, ATT_OP_READ_BY_GROUP_TYPE_REQ, 0, ATT_ERR_INVALID_PDU);
        }
        let start = u16::from_le_bytes([pdu[1], pdu[2]]);
        let end = u16::from_le_bytes([pdu[3], pdu[4]]);
        let group_uuid = u16::from_le_bytes([pdu[5], pdu[6]]);
        if group_uuid != GATT_UUID_PRIMARY_SERVICE {
            return self.write_att_error(
                out,
                ATT_OP_READ_BY_GROUP_TYPE_REQ,
                start,
                ATT_ERR_ATTRIBUTE_NOT_FOUND,
            );
        }

        // fixed 3 primary services: GAP, GATT, OTA(128-bit)
        // format: [opcode][len][start][end][uuid...]
        let mut count = 0usize;
        out[0] = ATT_OP_READ_BY_GROUP_TYPE_RSP;
        out[1] = 6;

        if start <= (OtaAttributeHandle::GapPrimaryService as u16)
            && end >= (OtaAttributeHandle::GapPrimaryService as u16)
            && out.len() >= 8
        {
            let off = 2 + count * 6;
            out[off..(off + 2)].copy_from_slice(&(OtaAttributeHandle::GapPrimaryService as u16).to_le_bytes());
            out[(off + 2)..(off + 4)]
                .copy_from_slice(&(OtaAttributeHandle::GapDeviceNameValue as u16).to_le_bytes());
            out[(off + 4)..(off + 6)].copy_from_slice(&0x1800u16.to_le_bytes());
            count += 1;
        }
        if start <= (OtaAttributeHandle::GattPrimaryService as u16)
            && end >= (OtaAttributeHandle::GattPrimaryService as u16)
            && out.len() >= 14
        {
            let off = 2 + count * 6;
            out[off..(off + 2)].copy_from_slice(&(OtaAttributeHandle::GattPrimaryService as u16).to_le_bytes());
            out[(off + 2)..(off + 4)]
                .copy_from_slice(&(OtaAttributeHandle::GattServiceChangedCcc as u16).to_le_bytes());
            out[(off + 4)..(off + 6)].copy_from_slice(&0x1801u16.to_le_bytes());
            count += 1;
        }

        if count == 0 {
            return self.write_att_error(
                out,
                ATT_OP_READ_BY_GROUP_TYPE_REQ,
                start,
                ATT_ERR_ATTRIBUTE_NOT_FOUND,
            );
        }
        2 + count * 6
    }

    fn handle_att_read_by_type_req(&mut self, pdu: &[u8], out: &mut [u8]) -> usize {
        if pdu.len() < 7 {
            return self.write_att_error(out, ATT_OP_READ_BY_TYPE_REQ, 0, ATT_ERR_INVALID_PDU);
        }
        let start = u16::from_le_bytes([pdu[1], pdu[2]]);
        let end = u16::from_le_bytes([pdu[3], pdu[4]]);
        let typ = u16::from_le_bytes([pdu[5], pdu[6]]);
        if typ != GATT_UUID_CHARACTER {
            return self.write_att_error(out, ATT_OP_READ_BY_TYPE_REQ, start, ATT_ERR_ATTRIBUTE_NOT_FOUND);
        }

        // [opcode][len][handle][prop][value_handle][uuid]
        out[0] = ATT_OP_READ_BY_TYPE_RSP;
        out[1] = 7;
        let mut count = 0usize;
        let entries = [
            (
                OtaAttributeHandle::GapDeviceNameChar as u16,
                CHAR_PROP_READ,
                OtaAttributeHandle::GapDeviceNameValue as u16,
                GATT_UUID_DEVICE_NAME,
            ),
            (
                OtaAttributeHandle::GattServiceChangedChar as u16,
                CHAR_PROP_INDICATE,
                OtaAttributeHandle::GattServiceChangedValue as u16,
                GATT_UUID_SERVICE_CHANGE,
            ),
        ];
        for entry in entries {
            if entry.0 < start || entry.0 > end {
                continue;
            }
            let off = 2 + count * 7;
            if out.len() < off + 7 {
                break;
            }
            out[off..(off + 2)].copy_from_slice(&entry.0.to_le_bytes());
            out[off + 2] = entry.1;
            out[(off + 3)..(off + 5)].copy_from_slice(&entry.2.to_le_bytes());
            out[(off + 5)..(off + 7)].copy_from_slice(&entry.3.to_le_bytes());
            count += 1;
        }
        if count == 0 {
            return self.write_att_error(out, ATT_OP_READ_BY_TYPE_REQ, start, ATT_ERR_ATTRIBUTE_NOT_FOUND);
        }
        2 + count * 7
    }

    fn handle_att_find_info_req(&mut self, pdu: &[u8], out: &mut [u8]) -> usize {
        if pdu.len() < 5 {
            return self.write_att_error(out, ATT_OP_FIND_INFO_REQ, 0, ATT_ERR_INVALID_PDU);
        }
        let start = u16::from_le_bytes([pdu[1], pdu[2]]);
        let end = u16::from_le_bytes([pdu[3], pdu[4]]);
        let entries = [
            (OtaAttributeHandle::GattServiceChangedCcc as u16, GATT_UUID_CLIENT_CHAR_CFG),
            (OtaAttributeHandle::OtaDataCcc as u16, GATT_UUID_CLIENT_CHAR_CFG),
            (OtaAttributeHandle::OtaDataUserDesc as u16, GATT_UUID_CHAR_USER_DESC),
        ];
        out[0] = ATT_OP_FIND_INFO_RSP;
        out[1] = 0x01;
        let mut count = 0usize;
        for (h, uuid) in entries {
            if h < start || h > end {
                continue;
            }
            let off = 2 + count * 4;
            if out.len() < off + 4 {
                break;
            }
            out[off..(off + 2)].copy_from_slice(&h.to_le_bytes());
            out[(off + 2)..(off + 4)].copy_from_slice(&uuid.to_le_bytes());
            count += 1;
        }
        if count == 0 {
            return self.write_att_error(out, ATT_OP_FIND_INFO_REQ, start, ATT_ERR_ATTRIBUTE_NOT_FOUND);
        }
        2 + count * 4
    }

    fn handle_att_read_req(&mut self, pdu: &[u8], out: &mut [u8]) -> usize {
        if pdu.len() < 3 {
            return self.write_att_error(out, ATT_OP_READ_REQ, 0, ATT_ERR_INVALID_PDU);
        }
        let handle = u16::from_le_bytes([pdu[1], pdu[2]]);
        out[0] = ATT_OP_READ_RSP;
        match handle {
            x if x == OtaAttributeHandle::GapDeviceNameValue as u16 => {
                let name = b"TLSR-OTA8";
                if out.len() < 1 + name.len() {
                    return 0;
                }
                out[1..(1 + name.len())].copy_from_slice(name);
                1 + name.len()
            }
            x if x == OtaAttributeHandle::GattServiceChangedValue as u16 => {
                if out.len() < 5 {
                    return 0;
                }
                out[1..5].copy_from_slice(&[0, 0, 0, 0]);
                5
            }
            x if x == OtaAttributeHandle::GattServiceChangedCcc as u16 => {
                if out.len() < 3 {
                    return 0;
                }
                out[1..3].copy_from_slice(&0u16.to_le_bytes());
                3
            }
            x if x == OtaAttributeHandle::OtaDataCcc as u16 => {
                if out.len() < 3 {
                    return 0;
                }
                let ccc = if self.status.ccc_enabled { 1u16 } else { 0u16 };
                out[1..3].copy_from_slice(&ccc.to_le_bytes());
                3
            }
            x if x == OtaAttributeHandle::OtaDataUserDesc as u16 => {
                let d = b"OTA";
                if out.len() < 1 + d.len() {
                    return 0;
                }
                out[1..(1 + d.len())].copy_from_slice(d);
                1 + d.len()
            }
            x if x == OtaAttributeHandle::OtaDataValue as u16 => {
                if out.len() < 2 {
                    return 0;
                }
                out[1] = 0;
                2
            }
            _ => self.write_att_error(out, ATT_OP_READ_REQ, handle, ATT_ERR_INVALID_HANDLE),
        }
    }

    fn handle_att_write_req(&mut self, pdu: &[u8], out: &mut [u8]) -> usize {
        if pdu.len() < 3 {
            return self.write_att_error(out, ATT_OP_WRITE_REQ, 0, ATT_ERR_INVALID_PDU);
        }
        let handle = u16::from_le_bytes([pdu[1], pdu[2]]);
        let value = &pdu[3..];
        match self.process_att_write(handle, value) {
            Ok(r) => {
                if let Some(pkt) = r.result_notify {
                    self.queue_notify(&pkt);
                } else if let Some(pkt) = r.schedule_notify {
                    self.queue_notify(&pkt);
                }
                out[0] = ATT_OP_WRITE_RSP;
                1
            }
            Err(e) => {
                let code = match e {
                    OtaCommandError::InvalidLength => ATT_ERR_INVALID_PDU,
                    OtaCommandError::InvalidFlow | OtaCommandError::InvalidIndex => ATT_ERR_WRITE_NOT_PERMITTED,
                    OtaCommandError::InvalidPduLength | OtaCommandError::InvalidSize => ATT_ERR_WRITE_NOT_PERMITTED,
                    OtaCommandError::FlashWrite => ATT_ERR_WRITE_NOT_PERMITTED,
                };
                self.write_att_error(out, ATT_OP_WRITE_REQ, handle, code)
            }
        }
    }

    fn handle_att_write_cmd(&mut self, pdu: &[u8]) {
        if pdu.len() < 3 {
            return;
        }
        let handle = u16::from_le_bytes([pdu[1], pdu[2]]);
        let value = &pdu[3..];
        if let Ok(r) = self.process_att_write(handle, value) {
            if let Some(pkt) = r.result_notify {
                self.queue_notify(&pkt);
            } else if let Some(pkt) = r.schedule_notify {
                self.queue_notify(&pkt);
            }
        }
    }

    fn write_att_error(&self, out: &mut [u8], req_opcode: u8, handle: u16, err: u8) -> usize {
        if out.len() < 5 {
            return 0;
        }
        out[0] = ATT_OP_ERROR_RSP;
        out[1] = req_opcode;
        out[2..4].copy_from_slice(&handle.to_le_bytes());
        out[4] = err;
        5
    }

    fn start_session(&mut self, pdu_len: u8) -> Result<(), OtaCommandError> {
        if pdu_len == 0 || pdu_len > 240 || (pdu_len & 0x0f) != 0 {
            self.status.ota_result = OTA_PACKET_INVALID;
            return Err(OtaCommandError::InvalidPduLength);
        }
        self.status.started = true;
        self.status.pdu_len = pdu_len;
        self.status.expected_index = 0;
        self.status.highest_written_index = 0;
        self.status.bytes_written = 0;
        self.status.ota_result = OTA_SUCCESS;
        self.erased_until = self.config.app_slot_base;
        Ok(())
    }

    fn handle_data_chunk(&mut self, data: &[u8]) -> Result<OtaProcessResult, OtaCommandError> {
        if data.len() < 3 {
            self.status.ota_result = OTA_PACKET_INVALID;
            return Err(OtaCommandError::InvalidLength);
        }
        if !self.status.started {
            self.status.ota_result = OTA_FLOW_ERR;
            return Err(OtaCommandError::InvalidFlow);
        }
        let idx = u16::from_le_bytes([data[0], data[1]]);
        if idx != self.status.expected_index {
            self.status.ota_result = OTA_PACKET_INVALID;
            return Err(OtaCommandError::InvalidIndex);
        }

        let payload = &data[2..];
        let chunk_len = payload.len() as u32;
        let offset = (idx as u32).saturating_mul(self.status.pdu_len as u32);
        if offset.saturating_add(chunk_len) > self.config.app_slot_size {
            self.status.ota_result = OTA_FW_SIZE_ERR;
            return Err(OtaCommandError::InvalidSize);
        }

        self.write_app_data(offset, payload)?;
        self.status.expected_index = self.status.expected_index.wrapping_add(1);
        self.status.highest_written_index = idx;
        let end = offset.saturating_add(chunk_len);
        if end > self.status.bytes_written {
            self.status.bytes_written = end;
        }

        let mut schedule = None;
        if self.status.ccc_enabled && ((idx as u32 + 1) % 16 == 0) {
            schedule = Some(build_schedule_fw_size(self.status.bytes_written));
        }
        Ok(OtaProcessResult {
            schedule_notify: schedule,
            result_notify: None,
        })
    }

    fn write_app_data(&mut self, offset: u32, payload: &[u8]) -> Result<(), OtaCommandError> {
        let addr = self.config.app_slot_base.saturating_add(offset);
        let needed_end = align_up(addr.saturating_add(payload.len() as u32), SECTOR_SIZE as u32);
        while self.erased_until < needed_end {
            self.flash.erase_sector(self.erased_until);
            self.erased_until = self.erased_until.saturating_add(SECTOR_SIZE as u32);
        }
        write_flash_bytes(&self.flash, addr, payload).map_err(|_| {
            self.status.ota_result = OTA_WRITE_FLASH_ERR;
            OtaCommandError::FlashWrite
        })
    }

    fn commit_image(&mut self) -> Result<(), OtaCommandError> {
        let meta = OtaImageMetadata {
            magic: OTA_META_MAGIC,
            state: OTA_META_STATE_READY,
            app_size: self.status.bytes_written,
            reserved: 0,
        };

        self.flash.erase_sector(self.config.metadata_addr);
        let raw = unsafe {
            core::slice::from_raw_parts(
                core::ptr::addr_of!(meta).cast::<u8>(),
                core::mem::size_of::<OtaImageMetadata>(),
            )
        };
        write_flash_bytes(&self.flash, self.config.metadata_addr, raw).map_err(|_| OtaCommandError::FlashWrite)
    }
}

fn write_flash_bytes(flash: &Flash, mut addr: u32, mut data: &[u8]) -> Result<(), ()> {
    while !data.is_empty() {
        let page_off = (addr as usize) & (PAGE_SIZE - 1);
        let room = PAGE_SIZE - page_off;
        let chunk = if data.len() < room { data.len() } else { room };
        flash.write_page(addr, &data[..chunk]);
        addr = addr.saturating_add(chunk as u32);
        data = &data[chunk..];
    }
    Ok(())
}

#[inline(always)]
fn align_up(value: u32, align: u32) -> u32 {
    if align == 0 {
        return value;
    }
    let rem = value % align;
    if rem == 0 {
        value
    } else {
        value + (align - rem)
    }
}

fn build_adv_data() -> ([u8; 31], usize) {
    let mut data = [0u8; 31];
    // Flags
    data[0] = 2;
    data[1] = 0x01;
    data[2] = 0x06;
    // Complete local name: "TLS"
    data[3] = 4;
    data[4] = 0x09;
    data[5] = b'T';
    data[6] = b'L';
    data[7] = b'S';
    // Manufacturer specific data: 0xFFFF + marker 'R'
    data[8] = 4;
    data[9] = 0xFF;
    data[10] = 0xFF;
    data[11] = 0xFF;
    data[12] = b'R';
    // Complete list of 128-bit service UUIDs
    data[13] = 17;
    data[14] = 0x07;
    let mut i = 0usize;
    while i < 16 {
        data[15 + i] = TELINK_OTA_SERVICE_UUID[i];
        i += 1;
    }
    (data, 31)
}

fn parse_connect_ind_summary(
    rx_meta: Option<BeaconRxMeta>,
    packet: &[u8],
) -> Option<OtaConnectIndSummary> {
    const BLE_PDU_CONNECT_IND: u8 = 0x05;
    let meta = rx_meta?;
    if meta.pdu_type != BLE_PDU_CONNECT_IND || !meta.target_match {
        return None;
    }
    // DMA packet: [0..1]=dma_len, [4]=header0, [5]=payload_len, [6..]=payload.
    // CONNECT_IND payload layout:
    // InitA(6), AdvA(6), AA(4), CRCInit(3), WinSize(1), WinOffset(2),
    // Interval(2), Latency(2), Timeout(2), ChM(5), Hop/SCA(1)
    if packet.len() < 40 || packet[5] < 34 {
        return None;
    }
    let payload = &packet[6..];
    if payload.len() < 34 {
        return None;
    }
    let mut init_addr_le = [0u8; 6];
    init_addr_le.copy_from_slice(&payload[0..6]);
    let access_addr = u32::from_le_bytes([payload[12], payload[13], payload[14], payload[15]]);
    let crc_init = [payload[16], payload[17], payload[18]];
    let channel_map = [payload[28], payload[29], payload[30], payload[31], payload[32]];
    let interval = u16::from_le_bytes([payload[22], payload[23]]);
    let latency = u16::from_le_bytes([payload[24], payload[25]]);
    let timeout = u16::from_le_bytes([payload[26], payload[27]]);
    let hop = payload[33] & 0x1f;
    Some(OtaConnectIndSummary {
        init_addr_le,
        access_addr,
        crc_init,
        channel_map,
        interval,
        latency,
        timeout,
        hop,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OtaConnDataReport {
    rx: bool,
    llid: u8,
    len: u8,
    ll_ctrl_opcode: u8,
    att_opcode: u8,
}

fn build_ll_att_tx_packet(att_pdu: &[u8], out: &mut [u8]) -> usize {
    // LL data PDU in RF DMA format:
    // [0..1]=dma_len, [4]=header0(LLID), [5]=len, [6..]=L2CAP(len+cid)+ATT
    if att_pdu.is_empty() || out.len() < 10 {
        return 0;
    }
    let l2cap_len = att_pdu.len();
    let payload_len = 4 + l2cap_len;
    let total = 6 + payload_len;
    if total > out.len() || payload_len > u8::MAX as usize {
        return 0;
    }
    out[..total].fill(0);
    let dma_len = (payload_len + 2) as u16;
    out[0] = (dma_len & 0xff) as u8;
    out[1] = (dma_len >> 8) as u8;
    out[4] = 0x02; // LLID=2 (start/complete L2CAP)
    out[5] = payload_len as u8;
    out[6..8].copy_from_slice(&(l2cap_len as u16).to_le_bytes());
    out[8..10].copy_from_slice(&0x0004u16.to_le_bytes()); // ATT CID
    out[10..(10 + l2cap_len)].copy_from_slice(att_pdu);
    total
}

fn parse_connection_data_report(
    connected: bool,
    schedule_armed: bool,
    rx_meta: Option<BeaconRxMeta>,
    packet: Option<&[u8]>,
) -> OtaConnDataReport {
    // Data-channel LLID is 2 bits in first header byte and valid only for data PDU types 0..3.
    if !connected || !schedule_armed {
        return OtaConnDataReport {
            rx: false,
            llid: 0,
            len: 0,
            ll_ctrl_opcode: 0,
            att_opcode: 0,
        };
    }
    let Some(meta) = rx_meta else {
        return OtaConnDataReport {
            rx: false,
            llid: 0,
            len: 0,
            ll_ctrl_opcode: 0,
            att_opcode: 0,
        };
    };
    if meta.pdu_type > 0x03 || meta.pdu_len == 0 {
        return OtaConnDataReport {
            rx: false,
            llid: 0,
            len: 0,
            ll_ctrl_opcode: 0,
            att_opcode: 0,
        };
    }
    let llid = meta.pdu_type & 0x03;
    let mut ll_ctrl_opcode = 0u8;
    let mut att_opcode = 0u8;
    if llid == 0x03 {
        if let Some(pkt) = packet {
            // LL Control PDU opcode is first byte of LL payload.
            if pkt.len() >= 7 {
                ll_ctrl_opcode = pkt[6];
            }
        }
    } else if llid == 0x02 {
        if let Some(pkt) = packet {
            // DMA packet: [0..1]=dma_len, [4]=LL header byte0, [5]=LL payload len, payload at [6..].
            // LL payload starts with L2CAP header: len(2), cid(2), then ATT payload for cid=0x0004.
            if pkt.len() >= 11 {
                let l2cap_len = u16::from_le_bytes([pkt[6], pkt[7]]) as usize;
                let cid = u16::from_le_bytes([pkt[8], pkt[9]]);
                if cid == 0x0004 && l2cap_len >= 1 {
                    att_opcode = pkt[10];
                }
            }
        }
    }
    OtaConnDataReport {
        rx: true,
        llid,
        len: meta.pdu_len,
        ll_ctrl_opcode,
        att_opcode,
    }
}

fn pick_data_channel(chm: [u8; 5], hop: u8, event_counter: u16) -> u8 {
    // BLE data channels are 0..36. Use CSA#1-style mapping:
    // unmapped = (counter * hop) % 37, then remap via used-channel list.
    let unmapped = ((event_counter as u32).wrapping_mul(hop as u32) % 37) as u8;
    if channel_enabled(chm, unmapped) {
        return unmapped;
    }
    let mut used = [0u8; 37];
    let mut n = 0usize;
    let mut ch = 0u8;
    while ch < 37 {
        if channel_enabled(chm, ch) {
            used[n] = ch;
            n += 1;
        }
        ch += 1;
    }
    if n == 0 {
        return 0;
    }
    used[(unmapped as usize) % n]
}

#[inline(always)]
fn conn_interval_us(interval_1_25ms: u16) -> u32 {
    // BLE interval unit is 1.25 ms.
    (interval_1_25ms as u32).saturating_mul(1250)
}

#[inline(always)]
fn channel_enabled(chm: [u8; 5], ch: u8) -> bool {
    if ch >= 37 {
        return false;
    }
    let byte = (ch >> 3) as usize;
    let bit = ch & 0x07;
    ((chm[byte] >> bit) & 1) != 0
}

#[inline(always)]
fn build_version_rsp(version: u16) -> [u8; 4] {
    let cmd = CMD_OTA_FW_VERSION_RSP.to_le_bytes();
    let v = version.to_le_bytes();
    [cmd[0], cmd[1], v[0], v[1]]
}

#[inline(always)]
fn build_result_notify(result: u8) -> [u8; 4] {
    let cmd = CMD_OTA_RESULT.to_le_bytes();
    [cmd[0], cmd[1], result, 0]
}

#[inline(always)]
fn build_schedule_fw_size(bytes: u32) -> [u8; 6] {
    let cmd = CMD_OTA_SCHEDULE_FW_SIZE.to_le_bytes();
    let sz = bytes.to_le_bytes();
    [cmd[0], cmd[1], sz[0], sz[1], sz[2], sz[3]]
}
