#include "tl_common.h"

startup_state_e drv_platform_init(void) {
    cpu_wakeup_init();
    clock_init(SYS_CLK_48M_Crystal);
    sysTimerPerUs = sys_tick_per_us;
    gpio_init(TRUE);
    return SYSTEM_BOOT;
}
