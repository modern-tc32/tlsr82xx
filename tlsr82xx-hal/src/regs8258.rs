//! Internal TLSR8258 register aliases.
//!
//! Names mirror `register.h`/related SDK headers where available. For a few
//! analog registers the SDK does not provide aliases; those keep neutral local
//! names or a clearly marked descriptive name.

pub(crate) const REG_MSPI_DATA: usize = 0x0080_000c;
pub(crate) const REG_MSPI_CTRL: usize = 0x0080_000d;

pub(crate) const REG_RST0: usize = 0x0080_0060;
pub(crate) const REG_RST1: usize = 0x0080_0061;
pub(crate) const REG_RST2: usize = 0x0080_0062;
pub(crate) const FLD_RST1_ZB: u8 = 1 << 0;
pub(crate) const REG_CLK_EN0: usize = 0x0080_0063;
pub(crate) const REG_CLK_EN1: usize = 0x0080_0064;
pub(crate) const REG_CLK_EN2: usize = 0x0080_0065;
pub(crate) const REG_CLK_SEL: usize = 0x0080_0066;
pub(crate) const REG_PWDN_CTRL: usize = 0x0080_006f;
pub(crate) const REG_ANA_POWER_CTRL: usize = 0x0080_0074;
pub(crate) const REG_MCU_WAKEUP_MASK: usize = 0x0080_0078;
pub(crate) const REG_PM_WAKEUP_FLAG: usize = 0x0080_007d;

pub(crate) const REG_IRQ_MASK: usize = 0x0080_0640;
pub(crate) const REG_IRQ_EN: usize = 0x0080_0643;
pub(crate) const REG_IRQ_SRC: usize = 0x0080_0648;
pub(crate) const REG_TMR_STA: usize = 0x0080_0623;
pub(crate) const REG_TMR0_TICK: usize = 0x0080_0630;
pub(crate) const REG_TMR1_TICK: usize = 0x0080_0634;
pub(crate) const REG_TMR2_TICK: usize = 0x0080_0638;

pub(crate) const REG_SYSTEM_TICK: usize = 0x0080_0740;
pub(crate) const REG_SYSTEM_TICK_CTRL: usize = 0x0080_074f;
pub(crate) const REG_DCDC_CTRL: usize = 0x0080_0750;

pub(crate) const REG_RF_IRQ_MASK: usize = 0x0080_0f1c;
pub(crate) const REG_RF_IRQ_STATUS: usize = 0x0080_0f20;
pub(crate) const REG_RF_LL_CTRL_0: usize = 0x0080_0f02;
pub(crate) const REG_RF_TX_SETTLE: usize = 0x0080_0f04;
pub(crate) const REG_RF_LL_CTRL_2: usize = 0x0080_0f15;
pub(crate) const REG_RF_LL_CTRL_3: usize = 0x0080_0f16;
pub(crate) const REG_GPIO_PE_IE: usize = 0x0080_05a1;
pub(crate) const REG_GPIO_WAKEUP_IRQ: usize = 0x0080_05b5;
pub(crate) const REG_DMA2_ADDR: usize = 0x0080_0c08;
pub(crate) const REG_DMA2_ADDR_HI: usize = 0x0080_0c42;
pub(crate) const REG_DFIFO2_ADDR: usize = 0x0080_0c08;
pub(crate) const REG_DFIFO2_SIZE: usize = 0x0080_0c0a;
pub(crate) const REG_DFIFO2_ADD_HI: usize = 0x0080_0c0b;
pub(crate) const REG_DFIFO_MODE: usize = 0x0080_0c10;
pub(crate) const REG_DFIFO0_ADDR: usize = 0x0080_0c40;
pub(crate) const REG_DFIFO1_ADDR: usize = 0x0080_0c44;
pub(crate) const REG_DFIFO0_SIZE: usize = 0x0080_0c48;
pub(crate) const REG_DMA_CHN_EN: usize = 0x0080_0c20;

// Used by vendor pm.o, but no alias was found in `register.h`.
pub(crate) const REG_WAKEUP_SRC: usize = 0x0080_0040;
pub(crate) const REG_PM_INFO0: usize = 0x0080_0048;
pub(crate) const REG_PM_INFO1: usize = 0x0080_004c;
pub(crate) const REG_SUSPEND_RET_ADDR_HI: usize = 0x0080_060d;

// RF helper aliases used by `rf_drv.h` as raw addresses without public names.
pub(crate) const REG_RF_MODE_CONTROL: usize = 0x0080_0f00;
pub(crate) const REG_RF_SN: usize = 0x0080_0f01;
pub(crate) const REG_RF_ACCESS_CODE: usize = 0x0080_0408;
pub(crate) const REG_RF_CHANNEL: usize = 0x0080_040d;
pub(crate) const REG_RF_RX_MODE: usize = 0x0080_0428;
pub(crate) const REG_RF_CRC: usize = 0x0080_044c;
pub(crate) const REG_RF_RSSI: usize = 0x0080_0441;
pub(crate) const REG_RF_RX_STATUS: usize = 0x0080_0448;
pub(crate) const REG_RF_POWER: usize = 0x0080_04a2;
pub(crate) const REG_PLL_RX_FINE_DIV_TUNE: usize = 0x0080_04d6;
pub(crate) const REG_DMA3_ADDR: usize = 0x0080_0c0c;
pub(crate) const REG_DMA3_ADDR_HI: usize = 0x0080_0c43;
pub(crate) const REG_RF_SCHED_TICK: usize = 0x0080_0f18;

pub(crate) const AREG_FLASH_VOLTAGE: u8 = 0x0c;
pub(crate) const AREG_CLK_SETTING: u8 = 0x82;
pub(crate) const AREG_ADC_SAMPLING_CLK_DIV: u8 = 0xf4;
pub(crate) const AREG_ADC_VREF: u8 = 0xe7;
pub(crate) const AREG_ADC_MISC_INPUT: u8 = 0xe8;
pub(crate) const AREG_ADC_RESOLUTION_MISC: u8 = 0xec;
pub(crate) const AREG_ADC_STATE_LENGTH_MC: u8 = 0xef;
pub(crate) const AREG_ADC_STATE_LENGTH_C: u8 = 0xf0;
pub(crate) const AREG_ADC_STATE_LENGTH_S: u8 = 0xf1;
pub(crate) const AREG_ADC_CHANNEL_ENABLE: u8 = 0xf2;
pub(crate) const AREG_ADC_VBAT_DIV: u8 = 0xf9;
pub(crate) const AREG_ADC_AIN_SCALE: u8 = 0xfa;
pub(crate) const AREG_ADC_PGA_BOOST: u8 = 0xfb;
pub(crate) const AREG_ADC_PGA_CTRL: u8 = 0xfc;
pub(crate) const AREG_ADC_MISC_L: u8 = 0xf7;
pub(crate) const AREG_ADC_MISC_H: u8 = 0xf8;
pub(crate) const FLD_CLK_24M_TO_SAR_EN: u8 = 1 << 5;
pub(crate) const FLD_RST1_ADC: u8 = 1 << 3;
pub(crate) const FLD_AUD_DFIFO2_IN: u8 = 1 << 2;

// Local descriptive names: vendor pm_get_32k_tick() reads them as one counter.
pub(crate) const ANA_32K_TICK_BYTE0: u8 = 0x40;
pub(crate) const ANA_32K_TICK_BYTE1: u8 = 0x41;
pub(crate) const ANA_32K_TICK_BYTE2: u8 = 0x42;
pub(crate) const ANA_32K_TICK_BYTE3: u8 = 0x43;

// Analog registers without SDK aliases confirmed for current use.
pub(crate) const ANA_REG_0X02: u8 = 0x02;
pub(crate) const ANA_REG_0X8A: u8 = 0x8a;
pub(crate) const ANA_REG_0X8C: u8 = 0x8c;
pub(crate) const ANA_REG_0X27: u8 = 0x27;
pub(crate) const ANA_REG_0X28: u8 = 0x28;
pub(crate) const ANA_REG_0X29: u8 = 0x29;
pub(crate) const ANA_REG_0X2A: u8 = 0x2a;

// Meanings inferred from `usbhw.h`.
pub(crate) const ANA_USB_DP_PULLUP: u8 = 0x0b;
pub(crate) const ANA_USB_POWER: u8 = 0x34;

pub(crate) const FLD_RF_IRQ_RX: u16 = 1 << 0;
pub(crate) const FLD_RF_IRQ_TX: u16 = 1 << 1;
pub(crate) const FLD_RF_IRQ_RX_TIMEOUT: u16 = 1 << 2;
pub(crate) const FLD_RF_IRQ_RX_CRC_2: u16 = 1 << 4;
pub(crate) const FLD_RF_IRQ_CMD_DONE: u16 = 1 << 5;
pub(crate) const FLD_RF_IRQ_FSM_TIMEOUT: u16 = 1 << 6;
pub(crate) const FLD_RF_IRQ_RETRY_HIT: u16 = 1 << 7;
pub(crate) const FLD_RF_IRQ_TX_DS: u16 = 1 << 8;
pub(crate) const FLD_RF_IRQ_RX_DR: u16 = 1 << 9;
pub(crate) const FLD_RF_IRQ_FIRST_TIMEOUT: u16 = 1 << 10;
pub(crate) const FLD_RF_IRQ_INVALID_PID: u16 = 1 << 11;
pub(crate) const FLD_RF_IRQ_STX_TIMEOUT: u16 = 1 << 12;
pub(crate) const FLD_RF_IRQ_ALL: u16 = 0x1fff;
