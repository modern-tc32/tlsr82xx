#include "tl_common.h"
#include "clock.h"
#include "gpio_default.h"
#include "irq.h"
#include "pm.h"
#include "timer.h"

u32 sysTimerPerUs;

startup_state_e drv_platform_init(void)
{
    drv_disable_irq();
    drv_irqMask_clear();

    cpu_wakeup_init();
    clock_init(SYS_CLK_48M_Crystal);
    sysTimerPerUs = sys_tick_per_us;
    gpio_init(1);
    drv_calibration();

    return SYSTEM_BOOT;
}

void drv_enable_irq(void)
{
    irq_enable();
}

u32 drv_disable_irq(void)
{
    return (u32)irq_disable();
}

void drv_irqMask_clear(void)
{
    irq_disable_type(FLD_IRQ_ALL);
}

u32 drv_restore_irq(u32 en)
{
    irq_restore((u8)en);
    return 0;
}
