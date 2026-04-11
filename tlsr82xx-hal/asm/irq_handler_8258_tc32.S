    .section .ram_code, "ax", %progbits
    .align 2
    .globl irq_handler
    .code 16
    .thumb_func
    .extern irq_handler_rust
    .extern tlsr82xx_timer0_irq_tick
    .extern tlsr82xx_system_timer_irq_service
    .extern TLSR82XX_TIMER0_IRQ_PERIOD_TICKS
    .extern TLSR82XX_SYSTEM_TIMER_IRQ_PERIOD_TICKS

irq_handler:
    tloadr r0, .Lirq_src
    tloadr r1, [r0]
    tmov r2, #1
    tmov r3, r1
    tand r3, r2
    tjne .Ltimer0_fast

    tloadr r2, .Lsystem_timer_bit
    tmov r3, r1
    tand r3, r2
    tjne .Lsystem_timer_fast

.Lgeneric:
    tpush {r14}
    tjl irq_handler_rust
    tpop {r0}
    tmov lr, r0
    tjex lr

.Ltimer0_fast:
    tloadr r0, .Lirq_src
    tmov r1, #1
    tstorer r1, [r0]

    tloadr r0, .Ltmr_sta
    tstorerb r1, [r0]

    tloadr r0, .Ltmr0_tick
    tmov r1, #0
    tstorer r1, [r0]

    tloadr r0, .Ltimer0_period_ptr
    tloadr r0, [r0]
    tloadr r1, .Ltmr0_capt
    tstorer r0, [r1]

    tloadr r0, .Ltmr_ctrl
    tloadr r1, [r0]
    tmov r2, #1
    tor r1, r2
    tstorer r1, [r0]

    tpush {r14}
    tjl tlsr82xx_timer0_irq_tick
    tpop {r0}
    tmov lr, r0
    tjex lr

.Lsystem_timer_fast:
    tpush {r14}
    tjl tlsr82xx_system_timer_irq_service
    tpop {r0}
    tmov lr, r0
    tjex lr

    .align 4
.Lirq_src:
    .word 0x00800648
.Ltmr_sta:
    .word 0x00800623
.Ltmr0_tick:
    .word 0x00800630
.Ltmr0_capt:
    .word 0x00800624
.Ltmr_ctrl:
    .word 0x00800620
.Ltimer0_period_ptr:
    .word TLSR82XX_TIMER0_IRQ_PERIOD_TICKS
.Lsystem_timer_bit:
    .word 0x00100000
