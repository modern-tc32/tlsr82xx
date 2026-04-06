#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdcReference {
    V0P6,
    V0P9,
    V1P2,
    VbatDivided,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdcVbatDivider {
    Off,
    Div1F4,
    Div1F3,
    Div1F2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdcResolution {
    Bits8,
    Bits10,
    Bits12,
    Bits14,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdcInputMode {
    SingleEnded,
    Differential,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdcSampleCycles {
    Cycles3,
    Cycles6,
    Cycles9,
    Cycles12,
    Cycles15,
    Cycles18,
    Cycles21,
    Cycles24,
    Cycles27,
    Cycles30,
    Cycles33,
    Cycles36,
    Cycles39,
    Cycles42,
    Cycles45,
    Cycles48,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdcChannel {
    Left,
    Right,
    Misc,
    Rns,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdcPrescaler {
    Div1,
    Div1F2,
    Div1F4,
    Div1F8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdcMode {
    Normal,
    Rns,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdcConfig {
    pub channel: AdcChannel,
    pub reference: AdcReference,
    pub vbat_divider: AdcVbatDivider,
    pub resolution: AdcResolution,
    pub sample_cycles: AdcSampleCycles,
    pub prescaler: AdcPrescaler,
    pub mode: AdcMode,
    pub input_mode: AdcInputMode,
}

impl AdcConfig {
    #[inline(always)]
    pub const fn gpio_single_ended() -> Self {
        Self {
            channel: AdcChannel::Misc,
            reference: AdcReference::V1P2,
            vbat_divider: AdcVbatDivider::Off,
            resolution: AdcResolution::Bits14,
            sample_cycles: AdcSampleCycles::Cycles6,
            prescaler: AdcPrescaler::Div1F8,
            mode: AdcMode::Normal,
            input_mode: AdcInputMode::Differential,
        }
    }

    #[inline(always)]
    pub const fn vbat() -> Self {
        Self {
            channel: AdcChannel::Misc,
            reference: AdcReference::V1P2,
            vbat_divider: AdcVbatDivider::Off,
            resolution: AdcResolution::Bits14,
            sample_cycles: AdcSampleCycles::Cycles6,
            prescaler: AdcPrescaler::Div1F8,
            mode: AdcMode::Normal,
            input_mode: AdcInputMode::Differential,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Adc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdcSample {
    pub millivolts: u32,
    pub fluctuation_mv: u32,
}

unsafe extern "C" {
    static mut adc_gpio_calib_vref: u16;

    fn adc_set_gpio_calib_vref(data: u16);
    fn adc_get_result_with_fluct(fluctuation_mv: *mut u32) -> u32;
}

impl Adc {
    #[inline(always)]
    pub const fn new() -> Self {
        Self
    }

    #[inline(always)]
    pub fn sample(&self) -> u32 {
        let mut fluctuation_mv = 0;
        // SAFETY: The backend symbol is provided by the linked runtime.
        unsafe { adc_get_result_with_fluct(&mut fluctuation_mv) }
    }

    #[inline(always)]
    pub fn sample_with_fluctuation(&self) -> AdcSample {
        let mut fluctuation_mv = 0;
        // SAFETY: The backend symbol is provided by the linked runtime.
        let millivolts = unsafe { adc_get_result_with_fluct(&mut fluctuation_mv) };
        AdcSample {
            millivolts,
            fluctuation_mv,
        }
    }

    #[inline(always)]
    pub fn gpio_calibration_vref_mv(&self) -> u16 {
        gpio_calibration_vref_mv()
    }

    #[inline(always)]
    pub fn set_gpio_calibration_vref_mv(&self, millivolts: u16) {
        set_gpio_calibration_vref_mv(millivolts);
    }

    #[inline(always)]
    pub const fn gpio_default_config(&self) -> AdcConfig {
        AdcConfig::gpio_single_ended()
    }

    #[inline(always)]
    pub const fn vbat_default_config(&self) -> AdcConfig {
        AdcConfig::vbat()
    }

    pub fn init_gpio_input(&self, pin: AdcGpioPin) {
        adc_common_init();
        adc_gpio_init(pin);
    }

    pub fn init_vbat_input(&self, pin: AdcGpioPin) {
        adc_common_init();
        adc_vbat_init(pin);
    }

    #[inline(always)]
    pub fn sample_current_config(&self) -> u32 {
        sample_current_config_with_fluctuation().millivolts
    }

    #[inline(always)]
    pub fn sample_current_config_with_fluctuation(&self) -> AdcSample {
        sample_current_config_with_fluctuation()
    }
}

#[inline(always)]
pub fn gpio_calibration_vref_mv() -> u16 {
    // SAFETY: Read-only access to a runtime calibration global.
    unsafe { adc_gpio_calib_vref }
}

#[inline(always)]
pub fn set_gpio_calibration_vref_mv(millivolts: u16) {
    // SAFETY: The runtime exposes this symbol for updating the calibration value.
    unsafe { adc_set_gpio_calib_vref(millivolts) }
}

#[inline(always)]
fn adc_reset_module() {
    unsafe {
        core::ptr::write_volatile(reg8(REG_RST1), FLD_RST1_ADC);
        core::ptr::write_volatile(reg8(REG_RST1), 0);
    }
}

#[inline(always)]
fn adc_enable_clk_24m(enable: bool) {
    let mut value = analog::read(crate::regs8258::AREG_CLK_SETTING);
    if enable {
        value |= FLD_CLK_24M_TO_SAR_EN;
    } else {
        value &= !FLD_CLK_24M_TO_SAR_EN;
    }
    analog::write(crate::regs8258::AREG_CLK_SETTING, value);
}

#[inline(always)]
fn adc_set_sample_clk(div: u8) {
    analog::write(AREG_ADC_SAMPLING_CLK_DIV, div & 0x07);
}

#[inline(always)]
fn adc_set_state_length(r_max_mc: u16, r_max_c: u16, r_max_s: u8) {
    analog::write(AREG_ADC_STATE_LENGTH_MC, r_max_mc as u8);
    analog::write(AREG_ADC_STATE_LENGTH_C, r_max_c as u8);
    analog::write(
        AREG_ADC_STATE_LENGTH_S,
        (((r_max_mc >> 8) as u8) << 6) | (((r_max_c >> 8) as u8) << 4) | (r_max_s & 0x0f),
    );
}

#[inline(always)]
fn adc_set_channel_enable_and_max_state_count(channel_bits: u8, count: u8) {
    analog::write(AREG_ADC_CHANNEL_ENABLE, channel_bits | ((count & 0x07) << 4));
}

#[inline(always)]
fn adc_config_misc_channel_buffer(buffer: *mut u16, size_bytes: usize) {
    let addr = buffer as usize;
    unsafe {
        core::ptr::write_volatile(reg16(REG_DFIFO2_ADDR), addr as u16);
        core::ptr::write_volatile(reg8(REG_DFIFO2_ADD_HI), (addr >> 16) as u8);
        core::ptr::write_volatile(reg8(REG_DFIFO2_SIZE), ((size_bytes >> 4) as u8).wrapping_sub(1));
    }
}

#[inline(always)]
fn dfifo_enable_dfifo2() {
    unsafe {
        let reg = reg8(REG_DFIFO_MODE);
        let value = core::ptr::read_volatile(reg.cast_const()) | FLD_AUD_DFIFO2_IN;
        core::ptr::write_volatile(reg, value);
    }
}

#[inline(always)]
fn dfifo_disable_dfifo2() {
    unsafe {
        let reg = reg8(REG_DFIFO_MODE);
        let value = core::ptr::read_volatile(reg.cast_const()) & !FLD_AUD_DFIFO2_IN;
        core::ptr::write_volatile(reg, value);
    }
}

#[allow(dead_code)]
#[inline(always)]
fn _adc_backend_touch() {
    let _ = (
        AREG_ADC_VREF,
        AREG_ADC_MISC_INPUT,
        AREG_ADC_RESOLUTION_MISC,
        AREG_ADC_VBAT_DIV,
        AREG_ADC_AIN_SCALE,
        AREG_ADC_PGA_BOOST,
        AREG_ADC_PGA_CTRL,
        AREG_ADC_MISC_L,
        AREG_ADC_MISC_H,
    );
    let _ = adc_reset_module as fn();
    let _ = adc_enable_clk_24m as fn(bool);
    let _ = adc_set_sample_clk as fn(u8);
    let _ = adc_set_state_length as fn(u16, u16, u8);
    let _ = adc_set_channel_enable_and_max_state_count as fn(u8, u8);
    let _ = adc_config_misc_channel_buffer as fn(*mut u16, usize);
    let _ = dfifo_enable_dfifo2 as fn();
    let _ = dfifo_disable_dfifo2 as fn();
}

#[inline(always)]
fn adc_common_init() {
    adc_power_on_sar_adc(false);
    adc_reset_module();
    adc_enable_clk_24m(true);
    adc_set_sample_clk(5);
    adc_power_on_pga_left(false);
    adc_power_on_pga_right(false);
    adc_set_gain_bias_per100();
    dfifo_disable_dfifo2();
    adc_set_state_length(1023, 0, 15);
}

#[inline(always)]
fn adc_gpio_init(pin: AdcGpioPin) {
    adc_set_channel_enable_and_max_state_count(1 << 2, 2);
    adc_set_reference_v1p2_misc();
    adc_set_vbat_divider(AdcVbatDivider::Off);
    adc_configure_base_pin(pin);
    adc_set_resolution_misc(AdcResolution::Bits14);
    adc_set_sample_cycles_misc(AdcSampleCycles::Cycles6);
    adc_set_prescaler(AdcPrescaler::Div1F8);
    adc_set_mode_raw(AdcMode::Normal);
}

#[inline(always)]
fn adc_vbat_init(pin: AdcGpioPin) {
    adc_set_channel_enable_and_max_state_count(1 << 2, 2);
    adc_set_vbat_divider(AdcVbatDivider::Off);
    adc_configure_vbat_pin(pin);
    adc_set_reference_v1p2_misc();
    adc_set_resolution_misc(AdcResolution::Bits14);
    adc_set_sample_cycles_misc(AdcSampleCycles::Cycles6);
    adc_set_prescaler(AdcPrescaler::Div1F8);
    adc_set_mode_raw(AdcMode::Normal);
}

#[inline(always)]
fn adc_configure_base_pin(pin: AdcGpioPin) {
    let raw = pin.raw_pin();
    gpio::set_function_raw(raw, PinFunction::Gpio);
    gpio::set_input_enabled_raw(raw, false);
    gpio::set_output_enabled_raw(raw, false);
    gpio::write_data_raw(raw, false);
    adc_set_misc_input_differential(pin.input_channel_index(), 0x0f);
}

#[inline(always)]
fn adc_configure_vbat_pin(pin: AdcGpioPin) {
    let raw = pin.raw_pin();
    gpio::set_function_raw(raw, PinFunction::Gpio);
    gpio::set_input_enabled_raw(raw, false);
    gpio::set_output_enabled_raw(raw, true);
    gpio::write_data_raw(raw, true);
    adc_set_misc_input_differential(pin.input_channel_index(), 0x0f);
}

#[inline(always)]
fn adc_power_on_sar_adc(on: bool) {
    let mut value = analog::read(AREG_ADC_PGA_CTRL);
    if on {
        value &= !(1 << 5);
    } else {
        value |= 1 << 5;
    }
    analog::write(AREG_ADC_PGA_CTRL, value);
}

#[inline(always)]
fn adc_power_on_pga_left(on: bool) {
    let mut value = analog::read(AREG_ADC_PGA_CTRL);
    if on {
        value &= !(1 << 6);
    } else {
        value |= 1 << 6;
    }
    analog::write(AREG_ADC_PGA_CTRL, value);
}

#[inline(always)]
fn adc_power_on_pga_right(on: bool) {
    let mut value = analog::read(AREG_ADC_PGA_CTRL);
    if on {
        value &= !(1 << 7);
    } else {
        value |= 1 << 7;
    }
    analog::write(AREG_ADC_PGA_CTRL, value);
}

#[inline(always)]
fn adc_set_gain_bias_per100() {
    let mut value = analog::read(AREG_ADC_PGA_CTRL);
    value &= !0x0f;
    value |= 0x01 | (0x01 << 2);
    analog::write(AREG_ADC_PGA_CTRL, value);
}

#[inline(always)]
fn adc_set_reference_v1p2_misc() {
    let mut value = analog::read(AREG_ADC_VREF);
    value = (value & !(0b11 << 4)) | (0x02 << 4);
    analog::write(AREG_ADC_VREF, value);
}

#[inline(always)]
fn adc_set_vbat_divider(divider: AdcVbatDivider) {
    let bits = match divider {
        AdcVbatDivider::Off => 0,
        AdcVbatDivider::Div1F4 => 1,
        AdcVbatDivider::Div1F3 => 2,
        AdcVbatDivider::Div1F2 => 3,
    };
    let mut value = analog::read(AREG_ADC_VBAT_DIV);
    value = (value & !(0b11 << 2)) | (bits << 2);
    analog::write(AREG_ADC_VBAT_DIV, value);
}

#[inline(always)]
fn adc_set_misc_input_differential(positive: u8, negative: u8) {
    analog::write(AREG_ADC_MISC_INPUT, (negative & 0x0f) | ((positive & 0x0f) << 4));
    let mut value = analog::read(AREG_ADC_RESOLUTION_MISC);
    value |= 1 << 6;
    analog::write(AREG_ADC_RESOLUTION_MISC, value);
}

#[inline(always)]
fn adc_set_resolution_misc(resolution: AdcResolution) {
    let bits = match resolution {
        AdcResolution::Bits8 => 0,
        AdcResolution::Bits10 => 1,
        AdcResolution::Bits12 => 2,
        AdcResolution::Bits14 => 3,
    };
    let mut value = analog::read(AREG_ADC_RESOLUTION_MISC);
    value = (value & !0b11) | bits;
    analog::write(AREG_ADC_RESOLUTION_MISC, value);
}

#[inline(always)]
fn adc_set_sample_cycles_misc(cycles: AdcSampleCycles) {
    let bits = match cycles {
        AdcSampleCycles::Cycles3 => 0,
        AdcSampleCycles::Cycles6 => 1,
        AdcSampleCycles::Cycles9 => 2,
        AdcSampleCycles::Cycles12 => 3,
        AdcSampleCycles::Cycles15 => 4,
        AdcSampleCycles::Cycles18 => 5,
        AdcSampleCycles::Cycles21 => 6,
        AdcSampleCycles::Cycles24 => 7,
        AdcSampleCycles::Cycles27 => 8,
        AdcSampleCycles::Cycles30 => 9,
        AdcSampleCycles::Cycles33 => 10,
        AdcSampleCycles::Cycles36 => 11,
        AdcSampleCycles::Cycles39 => 12,
        AdcSampleCycles::Cycles42 => 13,
        AdcSampleCycles::Cycles45 => 14,
        AdcSampleCycles::Cycles48 => 15,
    };
    let mut value = analog::read(AREG_ADC_MISC_H);
    value = (value & !0x0f) | bits;
    analog::write(AREG_ADC_MISC_H, value);
}

#[inline(always)]
fn adc_set_prescaler(prescaler: AdcPrescaler) {
    let bits = match prescaler {
        AdcPrescaler::Div1 => 0,
        AdcPrescaler::Div1F2 => 1,
        AdcPrescaler::Div1F4 => 2,
        AdcPrescaler::Div1F8 => 3,
    };
    let mut value = analog::read(AREG_ADC_AIN_SCALE);
    value = (value & !(0b11 << 6)) | (bits << 6);
    analog::write(AREG_ADC_AIN_SCALE, value);
}

#[inline(always)]
fn adc_set_mode_raw(mode: AdcMode) {
    let mut value = analog::read(AREG_ADC_PGA_CTRL);
    match mode {
        AdcMode::Normal => value &= !(1 << 4),
        AdcMode::Rns => value |= 1 << 4,
    }
    analog::write(AREG_ADC_PGA_CTRL, value);
}

const ADC_SAMPLE_NUM: usize = 8;
const ADC_WAIT_US_23K: u32 = 90;

#[repr(align(16))]
struct AlignedSampleBuffer([i32; ADC_SAMPLE_NUM]);

fn sample_current_config_with_fluctuation() -> AdcSample {
    let mut adc_data_buf = AlignedSampleBuffer([0; ADC_SAMPLE_NUM]);
    let mut adc_sample = [0u16; ADC_SAMPLE_NUM];

    adc_reset_module();
    adc_config_misc_channel_buffer(adc_data_buf.0.as_mut_ptr().cast::<u16>(), ADC_SAMPLE_NUM << 2);
    dfifo_enable_dfifo2();

    let mut t0 = crate::timer::clock_time();
    while !crate::timer::clock_time_exceed_us(t0, ADC_WAIT_US_23K) {}

    for i in 0..ADC_SAMPLE_NUM {
        while !crate::timer::clock_time_exceed_us(t0, ADC_WAIT_US_23K) {}
        t0 = crate::timer::clock_time();

        let sample = adc_data_buf.0[i];
        adc_sample[i] = if (sample & (1 << 13)) != 0 {
            0
        } else {
            (sample as u16) & 0x1fff
        };

        let mut j = i;
        while j > 0 && adc_sample[j] < adc_sample[j - 1] {
            adc_sample.swap(j, j - 1);
            j -= 1;
        }
    }

    dfifo_disable_dfifo2();

    let adc_average =
        (u32::from(adc_sample[2]) + u32::from(adc_sample[3]) + u32::from(adc_sample[4]) + u32::from(adc_sample[5]))
            / 4;

    if adc_average == 0 {
        return AdcSample {
            millivolts: 0,
            fluctuation_mv: 0,
        };
    }

    let adc_vref = u32::from(gpio_calibration_vref_mv());
    let adc_pre_scale = 8u32;
    let millivolts = ((adc_average * adc_pre_scale * adc_vref) >> 13) as u32;
    let fluctuation_mv =
        (((u32::from(adc_sample[ADC_SAMPLE_NUM - 1]) - u32::from(adc_sample[0])) * adc_pre_scale * adc_vref) >> 13)
            as u32;

    AdcSample {
        millivolts,
        fluctuation_mv,
    }
}
use crate::analog;
use crate::gpio::{self, PinFunction};
use crate::mmio::{reg16, reg8};
use crate::regs8258::{
    AREG_ADC_AIN_SCALE, AREG_ADC_CHANNEL_ENABLE, AREG_ADC_MISC_H, AREG_ADC_MISC_INPUT,
    AREG_ADC_MISC_L, AREG_ADC_PGA_BOOST, AREG_ADC_PGA_CTRL, AREG_ADC_RESOLUTION_MISC,
    AREG_ADC_SAMPLING_CLK_DIV, AREG_ADC_STATE_LENGTH_C, AREG_ADC_STATE_LENGTH_MC,
    AREG_ADC_STATE_LENGTH_S, AREG_ADC_VBAT_DIV, AREG_ADC_VREF, FLD_AUD_DFIFO2_IN,
    FLD_CLK_24M_TO_SAR_EN, FLD_RST1_ADC, REG_DFIFO2_ADDR, REG_DFIFO2_ADD_HI, REG_DFIFO2_SIZE,
    REG_DFIFO_MODE, REG_RST1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdcGpioPin {
    Pb0,
    Pb1,
    Pb2,
    Pb3,
    Pb4,
    Pb5,
    Pb6,
    Pb7,
    Pc4,
    Pc5,
}

impl AdcGpioPin {
    #[inline(always)]
    pub const fn raw_pin(self) -> u16 {
        match self {
            Self::Pb0 => 0x0101,
            Self::Pb1 => 0x0102,
            Self::Pb2 => 0x0104,
            Self::Pb3 => 0x0108,
            Self::Pb4 => 0x0110,
            Self::Pb5 => 0x0120,
            Self::Pb6 => 0x0140,
            Self::Pb7 => 0x0180,
            Self::Pc4 => 0x0210,
            Self::Pc5 => 0x0220,
        }
    }

    #[inline(always)]
    pub const fn input_channel_index(self) -> u8 {
        match self {
            Self::Pb0 => 1,
            Self::Pb1 => 2,
            Self::Pb2 => 3,
            Self::Pb3 => 4,
            Self::Pb4 => 5,
            Self::Pb5 => 6,
            Self::Pb6 => 7,
            Self::Pb7 => 8,
            Self::Pc4 => 9,
            Self::Pc5 => 10,
        }
    }
}
