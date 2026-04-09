use tlsr82xx_hal::radio::{BleConfig, IrqFlags, Radio, RadioError, RadioPower};
use tlsr82xx_hal::timer;

const PDU_TYPE_ADV_NONCONN_IND: u8 = 0x02;
const BLE_TX_ADDR_RANDOM_BIT: u8 = 1 << 6;
const BLE_ADV_HEADER0: u8 = PDU_TYPE_ADV_NONCONN_IND | BLE_TX_ADDR_RANDOM_BIT;
const TX_START_DELAY_US: u32 = 10;
const SUCCESS_IRQ_BITS: u16 = IrqFlags::TX.bits() | IrqFlags::TX_DS.bits() | IrqFlags::CMD_DONE.bits();
const TIMEOUT_IRQ_BITS: u16 = IrqFlags::STX_TIMEOUT.bits();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BeaconError {
    AdvDataTooLong,
    TxBufferTooSmall,
    Radio(RadioError),
}

impl From<RadioError> for BeaconError {
    fn from(value: RadioError) -> Self {
        Self::Radio(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BeaconFailureReason {
    Config,
    Timeout,
    UnexpectedIrq,
    StartTx,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BeaconEventResult {
    pub ok: bool,
    pub failed_channel: u8,
    pub tx_attempts: u8,
    pub tx_ok: u8,
    pub tx_timeout: u8,
    pub tx_other_irq: u8,
    pub last_irq: u16,
    pub failure_reason: Option<BeaconFailureReason>,
}

impl BeaconEventResult {
    const fn ok(tx_attempts: u8, tx_ok: u8, last_irq: u16) -> Self {
        Self {
            ok: true,
            failed_channel: 0,
            tx_attempts,
            tx_ok,
            tx_timeout: 0,
            tx_other_irq: 0,
            last_irq,
            failure_reason: None,
        }
    }

    const fn fail(
        failed_channel: u8,
        tx_attempts: u8,
        tx_ok: u8,
        tx_timeout: u8,
        tx_other_irq: u8,
        last_irq: u16,
        reason: BeaconFailureReason,
    ) -> Self {
        Self {
            ok: false,
            failed_channel,
            tx_attempts,
            tx_ok,
            tx_timeout,
            tx_other_irq,
            last_irq,
            failure_reason: Some(reason),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BeaconConfig {
    pub interval_us: u32,
    pub tx_timeout_us: u32,
    pub channel_settle_us: u32,
    pub channels: [u8; 3],
    pub power: RadioPower,
}

impl Default for BeaconConfig {
    fn default() -> Self {
        Self {
            interval_us: 100_000,
            tx_timeout_us: 4_000,
            channel_settle_us: 180,
            channels: [37, 38, 39],
            power: RadioPower::PLUS_10P46_DBM,
        }
    }
}

#[repr(align(4))]
struct Aligned<const N: usize>([u8; N]);

pub struct BeaconAdvertiser {
    radio: Radio,
    config: BeaconConfig,
    tx_packet: Aligned<64>,
    next_event_at: u32,
}

impl BeaconAdvertiser {
    pub fn new(
        radio: Radio,
        config: BeaconConfig,
        adv_addr_le: [u8; 6],
        adv_data: &[u8],
    ) -> Result<Self, BeaconError> {
        if adv_data.len() > 31 {
            return Err(BeaconError::AdvDataTooLong);
        }

        let pdu_payload_len = adv_addr_le.len() + adv_data.len();
        let pdu_len = 2 + pdu_payload_len;
        let dma_len = pdu_payload_len + 2;
        if dma_len + 4 > 64 {
            return Err(BeaconError::TxBufferTooSmall);
        }

        let mut tx_packet = Aligned([0u8; 64]);
        tx_packet.0[0] = (dma_len & 0xff) as u8;
        tx_packet.0[1] = ((dma_len >> 8) & 0xff) as u8;
        tx_packet.0[2] = 0;
        tx_packet.0[3] = 0;
        tx_packet.0[4] = BLE_ADV_HEADER0;
        tx_packet.0[5] = pdu_payload_len as u8;
        tx_packet.0[6..12].copy_from_slice(&adv_addr_le);
        tx_packet.0[12..(12 + adv_data.len())].copy_from_slice(adv_data);
        if pdu_len + 4 < tx_packet.0.len() {
            tx_packet.0[(pdu_len + 4)..].fill(0);
        }

        Ok(Self {
            radio,
            config,
            tx_packet,
            next_event_at: timer::clock_time().wrapping_sub(config.interval_us),
        })
    }

    #[inline(always)]
    fn adv_config(&self, channel: u8) -> BleConfig {
        BleConfig::advertising(channel).with_power(self.config.power)
    }

    pub fn init(&mut self) -> Result<(), BeaconError> {
        self.radio.init_mode(tlsr82xx_hal::radio::RadioMode::Ble1M)?;
        self.radio.apply_ble_config(self.adv_config(self.config.channels[0]))?;
        self.radio.enable_dma_tx_channel();
        self.radio.clear_all_irq_status();
        self.radio.clear_irq_mask(IrqFlags::ALL);
        self.radio
            .set_irq_mask(IrqFlags(SUCCESS_IRQ_BITS | TIMEOUT_IRQ_BITS));
        Ok(())
    }

    #[inline(always)]
    pub fn radio(&self) -> &Radio {
        &self.radio
    }

    #[inline(always)]
    pub fn radio_mut(&mut self) -> &mut Radio {
        &mut self.radio
    }

    #[inline(always)]
    pub fn should_run_event(&self) -> bool {
        timer::clock_time_exceed_us(self.next_event_at, self.config.interval_us)
    }

    pub fn run_event_if_due(&mut self) -> Option<BeaconEventResult> {
        if !self.should_run_event() {
            return None;
        }
        self.next_event_at = timer::clock_time();
        Some(self.run_event())
    }

    pub fn run_event(&mut self) -> BeaconEventResult {
        let mut tx_attempts = 0u8;
        let mut tx_ok = 0u8;
        let mut tx_timeout = 0u8;
        let mut tx_other_irq = 0u8;
        let mut last_irq = 0u16;

        for &channel in &self.config.channels {
            self.radio.set_tx_rx_off();
            if self.radio.apply_ble_config(self.adv_config(channel)).is_err() {
                return BeaconEventResult::fail(
                    channel,
                    tx_attempts,
                    tx_ok,
                    tx_timeout,
                    tx_other_irq.saturating_add(1),
                    last_irq,
                    BeaconFailureReason::Config,
                );
            }

            let settle_start = timer::clock_time();
            while !timer::clock_time_exceed_us(settle_start, self.config.channel_settle_us) {
                core::hint::spin_loop();
            }

            self.radio.clear_all_irq_status();
            tx_attempts = tx_attempts.wrapping_add(1);

            let tx_tick = timer::clock_time().wrapping_add(TX_START_DELAY_US);
            if self
                .radio
                .start_srx2tx_at(&self.tx_packet.0, tx_tick)
                .is_err()
            {
                return BeaconEventResult::fail(
                    channel,
                    tx_attempts,
                    tx_ok,
                    tx_timeout,
                    tx_other_irq.saturating_add(1),
                    last_irq,
                    BeaconFailureReason::StartTx,
                );
            }

            let wait_start = timer::clock_time();
            let mut channel_ok = false;
            while !timer::clock_time_exceed_us(wait_start, self.config.tx_timeout_us) {
                let irq = self.radio.irq_status().bits();
                last_irq = irq;
                if (irq & SUCCESS_IRQ_BITS) != 0 {
                    self.radio.clear_irq_status(IrqFlags(SUCCESS_IRQ_BITS));
                    tx_ok = tx_ok.wrapping_add(1);
                    channel_ok = true;
                    break;
                }
                if (irq & TIMEOUT_IRQ_BITS) != 0 {
                    self.radio.clear_irq_status(IrqFlags(TIMEOUT_IRQ_BITS));
                    tx_timeout = tx_timeout.wrapping_add(1);
                    return BeaconEventResult::fail(
                        channel,
                        tx_attempts,
                        tx_ok,
                        tx_timeout,
                        tx_other_irq,
                        irq,
                        BeaconFailureReason::Timeout,
                    );
                }
                if irq != 0 {
                    self.radio.clear_all_irq_status();
                    tx_other_irq = tx_other_irq.wrapping_add(1);
                    return BeaconEventResult::fail(
                        channel,
                        tx_attempts,
                        tx_ok,
                        tx_timeout,
                        tx_other_irq,
                        irq,
                        BeaconFailureReason::UnexpectedIrq,
                    );
                }
            }

            if !channel_ok {
                tx_timeout = tx_timeout.wrapping_add(1);
                return BeaconEventResult::fail(
                    channel,
                    tx_attempts,
                    tx_ok,
                    tx_timeout,
                    tx_other_irq,
                    last_irq,
                    BeaconFailureReason::Timeout,
                );
            }
        }

        BeaconEventResult::ok(tx_attempts, tx_ok, last_irq)
    }
}
