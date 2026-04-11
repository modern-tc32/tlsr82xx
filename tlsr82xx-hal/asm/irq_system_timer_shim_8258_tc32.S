    .section .ram_code, "ax", %progbits
    .align 2
    .globl irq_handler
    .code 16
    .thumb_func
    .extern tlsr82xx_system_timer_irq_tick
    .extern TLSR82XX_SYSTEM_TIMER_IRQ_PERIOD_TICKS

irq_handler:
    tloadr r0, .Lirq_src
    tloadr r1, .Lirq_bit
    tstorer r1, [r0]

    tloadr r0, .Lsystem_tick_irq
    tloadr r1, [r0]
    tloadr r2, .Lperiod_ptr
    tloadr r2, [r2]
    tadd r1, r2
    tstorer r1, [r0]

    tpush {r14}
    tjl tlsr82xx_system_timer_irq_tick
    tpop {r0}
    tmov lr, r0
    tjex lr

    .align 4
.Lirq_src:
    .word 0x00800648
.Lirq_bit:
    .word 0x00100000
.Lsystem_tick_irq:
    .word 0x00800744
.Lperiod_ptr:
    .word TLSR82XX_SYSTEM_TIMER_IRQ_PERIOD_TICKS
