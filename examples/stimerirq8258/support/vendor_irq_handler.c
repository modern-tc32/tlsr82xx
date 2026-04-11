#include <stdint.h>

#define REG_IRQ_SRC (*(volatile uint32_t *)0x00800648u)
#define REG_TMR_STA (*(volatile uint8_t *)0x00800623u)

#define FLD_IRQ_TMR0_EN 0x00000001u
#define FLD_IRQ_SYSTEM_TIMER 0x00100000u
#define FLD_TMR_STA_TMR0 0x01u

extern void tlsr82xx_timer0_irq_tick(void);
extern void tlsr82xx_system_timer_irq_service(void);

__attribute__((section(".ram_code")))
void irq_handler(void) {
    uint32_t src = REG_IRQ_SRC;

    if (src & FLD_IRQ_TMR0_EN) {
        REG_IRQ_SRC = FLD_IRQ_TMR0_EN;
        REG_TMR_STA = FLD_TMR_STA_TMR0;
        tlsr82xx_timer0_irq_tick();
    }

    if (src & FLD_IRQ_SYSTEM_TIMER) {
        REG_IRQ_SRC = FLD_IRQ_SYSTEM_TIMER;
        tlsr82xx_system_timer_irq_service();
    }
}
