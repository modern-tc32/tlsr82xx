//! Internal TLSR8258 register aliases.
//!
//! Names mirror `register.h`/related SDK headers where available. For a few
//! analog registers the SDK does not provide aliases; those keep neutral local
//! names or a clearly marked descriptive name.

pub(crate) const REG_BASE_ADDR: usize = 0x0080_0000;

pub(crate) const REG_I2C_SPEED: usize = 0x0080_0000;
pub(crate) const REG_I2C_ID: usize = 0x0080_0001;
pub(crate) const REG_I2C_STATUS: usize = 0x0080_0002;
pub(crate) const REG_I2C_MODE: usize = 0x0080_0003;
pub(crate) const REG_I2C_DO: usize = 0x0080_0005;
pub(crate) const REG_I2C_DI: usize = 0x0080_0006;
pub(crate) const REG_I2C_CTRL: usize = 0x0080_0007;
pub(crate) const FLD_I2C_WRITE_READ_BIT: u8 = 1 << 0;
pub(crate) const FLD_I2C_CMD_BUSY: u8 = 1 << 0;
pub(crate) const FLD_I2C_NAK: u8 = 1 << 2;
pub(crate) const FLD_I2C_MASTER_EN: u8 = 1 << 1;
pub(crate) const FLD_I2C_HOLD_MASTER: u8 = 1 << 3;
pub(crate) const FLD_I2C_CMD_ID: u8 = 1 << 0;
pub(crate) const FLD_I2C_CMD_DO: u8 = 1 << 2;
pub(crate) const FLD_I2C_CMD_DI: u8 = 1 << 3;
pub(crate) const FLD_I2C_CMD_START: u8 = 1 << 4;
pub(crate) const FLD_I2C_CMD_STOP: u8 = 1 << 5;
pub(crate) const FLD_I2C_CMD_READ_ID: u8 = 1 << 6;
pub(crate) const FLD_I2C_CMD_ACK: u8 = 1 << 7;

#[allow(dead_code)]
pub(crate) const REG_SPI_DATA: usize = 0x0080_0008;
pub(crate) const REG_SPI_CTRL: usize = 0x0080_0009;
pub(crate) const REG_SPI_SP: usize = 0x0080_000a;
pub(crate) const REG_SPI_INV_CLK: usize = 0x0080_000b;
pub(crate) const REG_MSPI_DATA: usize = 0x0080_000c;
pub(crate) const REG_MSPI_CTRL: usize = 0x0080_000d;
pub(crate) const FLD_SPI_ENABLE: u8 = 1 << 7;
pub(crate) const FLD_SPI_MASTER_MODE_EN: u8 = 1 << 1;
pub(crate) const FLD_SPI_DATA_OUT_DIS: u8 = 1 << 2;
pub(crate) const FLD_SPI_RD: u8 = 1 << 3;
pub(crate) const FLD_SPI_SHARE_MODE: u8 = 1 << 5;
pub(crate) const FLD_SPI_BUSY: u8 = 1 << 6;

pub(crate) const REG_WAKEUP_SRC: usize = 0x0080_0040;
pub(crate) const REG_PM_INFO0: usize = 0x0080_0048;
pub(crate) const REG_PM_INFO1: usize = 0x0080_004c;

pub(crate) const REG_RST0: usize = 0x0080_0060;
pub(crate) const REG_RST1: usize = 0x0080_0061;
pub(crate) const REG_RST2: usize = 0x0080_0062;
pub(crate) const FLD_RST0_I2C: u8 = 1 << 1;
pub(crate) const FLD_RST1_ZB: u8 = 1 << 0;
pub(crate) const FLD_RST1_ADC: u8 = 1 << 5;

pub(crate) const REG_CLK_EN0: usize = 0x0080_0063;
pub(crate) const REG_CLK_EN1: usize = 0x0080_0064;
pub(crate) const REG_CLK_EN2: usize = 0x0080_0065;
pub(crate) const FLD_CLK0_I2C_EN: u8 = 1 << 1;
pub(crate) const FLD_CLK0_SPI_EN: u8 = 1 << 0;
pub(crate) const REG_CLK_SEL: usize = 0x0080_0066;
pub(crate) const REG_PWDN_CTRL: usize = 0x0080_006f;
pub(crate) const REG_ANA_POWER_CTRL: usize = 0x0080_0074;
pub(crate) const REG_MCU_WAKEUP_MASK: usize = 0x0080_0078;
pub(crate) const REG_PM_WAKEUP_FLAG: usize = 0x0080_007d;

pub(crate) const REG_GPIO_PA_IE: usize = 0x0080_0581;
pub(crate) const REG_GPIO_PA_OEN: usize = 0x0080_0582;
pub(crate) const REG_GPIO_PA_GPIO: usize = 0x0080_0586;
pub(crate) const REG_GPIO_PC_GPIO: usize = 0x0080_0596;
pub(crate) const REG_GPIO_PE_IE: usize = 0x0080_05a1;
pub(crate) const REG_MUX_FUNC_A2: usize = 0x0080_05a9;
pub(crate) const REG_GPIO_WAKEUP_IRQ: usize = 0x0080_05b5;

pub(crate) const REG_TMR_CTRL: usize = 0x0080_0620;
pub(crate) const REG_TMR_STA: usize = 0x0080_0623;
pub(crate) const REG_TMR0_CAPT: usize = 0x0080_0624;
pub(crate) const REG_TMR0_TICK: usize = 0x0080_0630;
pub(crate) const REG_TMR1_TICK: usize = 0x0080_0634;
pub(crate) const REG_TMR2_TICK: usize = 0x0080_0638;
pub(crate) const FLD_TMR0_EN: u32 = 1 << 0;
pub(crate) const FLD_TMR0_MODE: u32 = 0b11 << 1;
pub(crate) const FLD_TMR_STA_TMR0: u8 = 1 << 0;
pub(crate) const FLD_TMR_STA_TMR1: u8 = 1 << 1;
pub(crate) const FLD_TMR_STA_TMR2: u8 = 1 << 2;

pub(crate) const REG_TL_MULTI_ADDR: usize = 0x0080_063e;
pub(crate) const REG_IRQ_MASK: usize = 0x0080_0640;
pub(crate) const REG_IRQ_EN: usize = 0x0080_0643;
pub(crate) const REG_IRQ_SRC: usize = 0x0080_0648;
pub(crate) const FLD_IRQ_TMR0_EN: u32 = 1 << 0;
pub(crate) const FLD_IRQ_TMR1_EN: u32 = 1 << 1;
pub(crate) const FLD_IRQ_TMR2_EN: u32 = 1 << 2;
pub(crate) const FLD_IRQ_GPIO_EN: u32 = 1 << 18;
pub(crate) const FLD_IRQ_SYSTEM_TIMER: u32 = 1 << 20;
pub(crate) const FLD_IRQ_GPIO_RISC0_EN: u32 = 1 << 21;
pub(crate) const FLD_IRQ_GPIO_RISC1_EN: u32 = 1 << 22;

pub(crate) const REG_PWM_ENABLE: usize = 0x0080_0780;
pub(crate) const REG_PWM_CLK: usize = 0x0080_0782;
pub(crate) const REG_PWM1_CMP: usize = 0x0080_0798;
pub(crate) const REG_PWM1_MAX: usize = 0x0080_079a;

pub(crate) const REG_SYSTEM_TICK: usize = 0x0080_0740;
pub(crate) const REG_SYSTEM_TICK_IRQ: usize = 0x0080_0744;
pub(crate) const REG_SYSTEM_WAKEUP_TICK: usize = 0x0080_0748;
pub(crate) const REG_SYSTEM_TICK_MODE: usize = 0x0080_074c;
pub(crate) const REG_SYSTEM_TICK_CTRL: usize = 0x0080_074f;
pub(crate) const REG_SYSTEM_32K_TICK_RD: usize = 0x0080_0750;
pub(crate) const REG_SYSTEM_32K_TICK_CAL: usize = 0x0080_0754;

pub(crate) const REG_DFIFO2_ADDR: usize = 0x0080_0c08;
pub(crate) const REG_DFIFO2_SIZE: usize = 0x0080_0c0a;
pub(crate) const REG_DFIFO2_ADD_HI: usize = 0x0080_0c0b;
pub(crate) const REG_DMA3_ADDR: usize = 0x0080_0c0c;
pub(crate) const REG_DFIFO_MODE: usize = 0x0080_0c10;
pub(crate) const FLD_AUD_DFIFO2_IN: u8 = 1 << 2;
pub(crate) const REG_DMA_CHN_EN: usize = 0x0080_0c20;
pub(crate) const REG_DMA_TX_RDY0: usize = 0x0080_0c24;
pub(crate) const REG_DFIFO0_ADDR: usize = 0x0080_0c40;
pub(crate) const REG_DMA2_ADDR_HI: usize = 0x0080_0c42;
pub(crate) const REG_DMA3_ADDR_HI: usize = 0x0080_0c43;
pub(crate) const REG_DFIFO1_ADDR: usize = 0x0080_0c44;
pub(crate) const REG_DFIFO0_SIZE: usize = 0x0080_0c48;
pub(crate) const REG_DMA2_ADDR: usize = 0x0080_0c08;
pub(crate) const REG_PM_RET_SRAM_CTRL: usize = 0x0080_0602;
pub(crate) const REG_SUSPEND_RET_ADDR_HI: usize = 0x0080_060d;

// RF helper aliases used by `rf_drv.h` as raw addresses without public names.
pub(crate) const REG_RF_ACCESS_CODE: usize = 0x0080_0408;
pub(crate) const REG_RF_CHANNEL: usize = 0x0080_040d;
pub(crate) const REG_RF_RSSI: usize = 0x0080_0441;
pub(crate) const REG_RF_RX_STATUS: usize = 0x0080_0448;
pub(crate) const REG_RF_CRC: usize = 0x0080_044c;
pub(crate) const REG_RF_POWER: usize = 0x0080_04a2;
pub(crate) const REG_PLL_RX_FINE_DIV_TUNE: usize = 0x0080_04d6;
pub(crate) const REG_RF_MODE_CONTROL: usize = 0x0080_0f00;
pub(crate) const REG_RF_SN: usize = 0x0080_0f01;
pub(crate) const REG_RF_LL_CTRL_0: usize = 0x0080_0f02;
pub(crate) const REG_RF_TX_SETTLE: usize = 0x0080_0f04;
pub(crate) const REG_RF_LL_CTRL_2: usize = 0x0080_0f15;
pub(crate) const REG_RF_LL_CTRL_3: usize = 0x0080_0f16;
pub(crate) const REG_RF_SCHED_TICK: usize = 0x0080_0f18;
pub(crate) const REG_RF_IRQ_MASK: usize = 0x0080_0f1c;
pub(crate) const REG_RF_IRQ_STATUS: usize = 0x0080_0f20;
pub(crate) const REG_RF_RX_MODE: usize = 0x0080_0428;
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

// Analog register names follow TLSR8258 vendor headers and datasheet mnemonics
// where those names are explicitly available. Raw AREG_0X.. constants are kept
// only for addresses that are still unnamed in the published references.
pub(crate) const AREG_DIG_LDO_CAP: u8 = 0x01;
pub(crate) const AREG_CLK_2M_RC: u8 = 0x02;
pub(crate) const AREG_0X03: u8 = 0x03;
pub(crate) const AREG_0X04: u8 = 0x04;
pub(crate) const AREG_PWDN_SETTING1: u8 = 0x05;
pub(crate) const AREG_PLL_BG: u8 = 0x06;
pub(crate) const AREG_LDO_SETTING1: u8 = 0x07;
// Vendor headers use `areg_dcdc_ctrl`; datasheet also documents USB DP pull-up
// and comparator control bits at this address.
pub(crate) const AREG_DCDC_CTRL: u8 = 0x0b;
pub(crate) const AREG_PA0_PA3_PULL: u8 = 0x0e;
pub(crate) const AREG_PA4_PA7_PULL: u8 = 0x0f;
pub(crate) const AREG_PB0_PB3_PULL: u8 = 0x10;
pub(crate) const AREG_PB4_PB7_PULL: u8 = 0x11;
pub(crate) const AREG_PC0_PC3_PULL: u8 = 0x12;
pub(crate) const AREG_PC4_PC7_PULL: u8 = 0x13;
pub(crate) const AREG_PD0_PD3_PULL: u8 = 0x14;
pub(crate) const AREG_PD4_PD7_PULL: u8 = 0x15;
pub(crate) const AREG_FLASH_VOLTAGE: u8 = 0x0c;
pub(crate) const AREG_R_DLY1: u8 = 0x1f;
pub(crate) const AREG_0X20: u8 = 0x20;
pub(crate) const AREG_WAKEUP_EN: u8 = 0x26;
pub(crate) const AREG_GPIO_WAKEUP_EN_PA: u8 = 0x27;
pub(crate) const AREG_GPIO_WAKEUP_EN_PB: u8 = 0x28;
pub(crate) const AREG_GPIO_WAKEUP_EN_PC: u8 = 0x29;
pub(crate) const AREG_GPIO_WAKEUP_EN_PD: u8 = 0x2a;
pub(crate) const AREG_0X2B: u8 = 0x2b;
pub(crate) const AREG_0X2C: u8 = 0x2c;
pub(crate) const AREG_0X2D: u8 = 0x2d;
pub(crate) const AREG_0X30: u8 = 0x30;
pub(crate) const AREG_0X31: u8 = 0x31;
pub(crate) const AREG_0X32: u8 = 0x32;
pub(crate) const AREG_0X33: u8 = 0x33;
pub(crate) const AREG_PWDN_SETTING: u8 = 0x34;

// Analog registers below can store data in deepsleep mode or deepsleep with
// SRAM retention mode. They are reset by watchdog, chip reset, RESET pin, and
// power cycle.
#[allow(dead_code)]
pub(crate) const AREG_DEEP6: u8 = 0x35; // initial value = 0x20
#[allow(dead_code)]
pub(crate) const AREG_DEEP7: u8 = 0x36; // initial value = 0x00
#[allow(dead_code)]
pub(crate) const AREG_DEEP8: u8 = 0x37; // initial value = 0x00
#[allow(dead_code)]
pub(crate) const AREG_DEEP9: u8 = 0x38; // initial value = 0x00
#[allow(dead_code)]
pub(crate) const AREG_DEEP10: u8 = 0x39; // initial value = 0xff

// Analog registers below can store information when MCU is in deepsleep mode:
// write before deepsleep, then read after wakeup. They are reset only by power
// cycle.
#[allow(dead_code)]
pub(crate) const AREG_DEEP0: u8 = 0x3a; // initial value = 0x00
#[allow(dead_code)]
pub(crate) const AREG_DEEP1: u8 = 0x3b; // initial value = 0x00
pub(crate) const AREG_DEEP2: u8 = 0x3c; // initial value = 0x00

pub(crate) const AREG_32K_TICK_0: u8 = 0x40;
pub(crate) const AREG_32K_TICK_1: u8 = 0x41;
pub(crate) const AREG_32K_TICK_2: u8 = 0x42;
pub(crate) const AREG_32K_TICK_3: u8 = 0x43;
pub(crate) const AREG_WAKEUP_STATUS: u8 = 0x44;
pub(crate) const AREG_0X7E: u8 = 0x7e;
pub(crate) const AREG_PM_STATUS: u8 = 0x7f;
pub(crate) const AREG_CLK_SETTING: u8 = 0x82;
pub(crate) const FLD_CLK_24M_TO_SAR_EN: u8 = 1 << 6;
pub(crate) const AREG_0X86: u8 = 0x86;
pub(crate) const AREG_0X87: u8 = 0x87;
pub(crate) const AREG_0X88: u8 = 0x88;
pub(crate) const AREG_XO_SETTING: u8 = 0x8a;
pub(crate) const AREG_LDO_TRIM: u8 = 0x8c;
pub(crate) const AREG_GPIO_PB_IE: u8 = 0xbd;
pub(crate) const AREG_GPIO_PB_DS: u8 = 0xbf;
pub(crate) const AREG_GPIO_PC_IE: u8 = 0xc0;
pub(crate) const AREG_GPIO_PC_DS: u8 = 0xc2;
pub(crate) const AREG_0XC6: u8 = 0xc6;
pub(crate) const AREG_0XC7: u8 = 0xc7;
pub(crate) const AREG_0XC8: u8 = 0xc8;
pub(crate) const AREG_0XC9: u8 = 0xc9;
pub(crate) const AREG_0XCA: u8 = 0xca;
pub(crate) const AREG_0XCB: u8 = 0xcb;
pub(crate) const AREG_0XCF: u8 = 0xcf;

pub(crate) const AREG_ADC_VREF: u8 = 0xe7;
pub(crate) const AREG_ADC_AIN_CHN_MISC: u8 = 0xe8;
pub(crate) const AREG_ADC_RES_M: u8 = 0xec;
pub(crate) const AREG_R_MAX_MC: u8 = 0xef;
pub(crate) const AREG_R_MAX_C: u8 = 0xf0;
pub(crate) const AREG_R_MAX_S: u8 = 0xf1;
pub(crate) const AREG_ADC_CHN_EN: u8 = 0xf2;
pub(crate) const AREG_ADC_SAMPLING_CLK_DIV: u8 = 0xf4;
pub(crate) const AREG_ADC_MISC_L: u8 = 0xf7;
pub(crate) const AREG_ADC_MISC_H: u8 = 0xf8;
pub(crate) const AREG_ADC_VREF_VBAT_DIV: u8 = 0xf9;
pub(crate) const AREG_AIN_SCALE: u8 = 0xfa;
pub(crate) const AREG_ADC_PGA_BOOST: u8 = 0xfb;
pub(crate) const AREG_ADC_PGA_CTRL: u8 = 0xfc;
