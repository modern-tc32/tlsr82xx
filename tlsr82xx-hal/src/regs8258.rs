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
pub(crate) const REG_GPIO_PE_IE: usize = 0x0080_05a1;
pub(crate) const REG_GPIO_WAKEUP_IRQ: usize = 0x0080_05b5;
pub(crate) const REG_DFIFO0_ADDR: usize = 0x0080_0c40;
pub(crate) const REG_DFIFO1_ADDR: usize = 0x0080_0c44;
pub(crate) const REG_DFIFO0_SIZE: usize = 0x0080_0c48;
pub(crate) const REG_DMA_CHN_EN: usize = 0x0080_0c20;

// Used by vendor pm.o, but no alias was found in `register.h`.
pub(crate) const REG_WAKEUP_SRC: usize = 0x0080_0040;
pub(crate) const REG_PM_INFO0: usize = 0x0080_0048;
pub(crate) const REG_PM_INFO1: usize = 0x0080_004c;
pub(crate) const REG_SUSPEND_RET_ADDR_HI: usize = 0x0080_060d;

pub(crate) const AREG_FLASH_VOLTAGE: u8 = 0x0c;
pub(crate) const AREG_CLK_SETTING: u8 = 0x82;

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
