use crate::mmio::{reg16, reg8};
use crate::regs8258::{
    FLD_RF_IRQ_ALL, FLD_RF_IRQ_CMD_DONE, FLD_RF_IRQ_FIRST_TIMEOUT, FLD_RF_IRQ_FSM_TIMEOUT,
    FLD_RF_IRQ_INVALID_PID, FLD_RF_IRQ_RETRY_HIT, FLD_RF_IRQ_RX, FLD_RF_IRQ_RX_CRC_2,
    FLD_RF_IRQ_RX_DR, FLD_RF_IRQ_RX_TIMEOUT, FLD_RF_IRQ_STX_TIMEOUT, FLD_RF_IRQ_TX,
    FLD_RF_IRQ_TX_DS, FLD_RST1_ZB, REG_DMA2_ADDR, REG_DMA2_ADDR_HI, REG_RF_AUTO_MODE,
    REG_RF_IRQ_MASK, REG_RF_IRQ_STATUS, REG_RF_LL_CTRL_0, REG_RF_LL_CTRL_2, REG_RF_LL_CTRL_3,
    REG_RF_POWER, REG_RF_RSSI, REG_RF_RX_MODE, REG_RF_RX_STATUS, REG_RF_SN, REG_RF_TX_SETTLE,
    REG_RST1,
};

const RF_TRX_MODE: u8 = 0xe0;
const RF_TRX_OFF: u8 = 0x45;

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
            core::ptr::write_volatile(reg8(REG_RF_AUTO_MODE), 0x80);
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
        unsafe {
            core::ptr::write_volatile(reg16(REG_RF_TX_SETTLE), settle_us);
        }
    }

    #[inline(always)]
    pub fn set_rx_buffer(&mut self, buffer: *mut u8) {
        let addr = buffer as usize;
        unsafe {
            core::ptr::write_volatile(reg16(REG_DMA2_ADDR), addr as u16);
            core::ptr::write_volatile(reg8(REG_DMA2_ADDR_HI), (addr >> 16) as u8);
        }
    }

    #[inline(always)]
    pub fn set_power_raw(&mut self, value: u8) {
        unsafe {
            core::ptr::write_volatile(reg8(REG_RF_POWER), value);
        }
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
