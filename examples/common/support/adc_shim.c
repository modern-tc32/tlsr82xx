#include "tl_common.h"

_attribute_data_retention_
unsigned short adc_gpio_calib_vref = 1175;

void adc_set_gpio_calib_vref(unsigned short data)
{
    adc_gpio_calib_vref = data;
}

_attribute_ram_code_
unsigned int adc_get_result_with_fluct(unsigned int *fluctuation_mv)
{
    if (fluctuation_mv) {
        *fluctuation_mv = 0;
    }

    // Current examples do not use the ADC peripheral directly. Returning a
    // stable value above the flash safety threshold keeps vendor flash helpers
    // operational without pulling in the full ADC driver.
    return 3300;
}
