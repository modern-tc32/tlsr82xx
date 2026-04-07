    .syntax unified
    .thumb

    .section .ram_code, "ax", %progbits
    .align 2
    .globl irq_handler
    .thumb_func
    .extern tlsr82xx_timer0_irq_tick
    .extern TLSR82XX_TIMER0_IRQ_PERIOD_TICKS
irq_handler:
    ldr r0, =0x00800648
    movs r1, #1
    str r1, [r0]

    ldr r0, =0x00800623
    strb r1, [r0]

    ldr r0, =0x00800630
    movs r1, #0
    str r1, [r0]

    ldr r0, =TLSR82XX_TIMER0_IRQ_PERIOD_TICKS
    ldr r0, [r0]
    ldr r1, =0x00800624
    str r0, [r1]

    ldr r0, =0x00800620
    ldr r1, [r0]
    movs r2, #1
    orrs r1, r2
    str r1, [r0]

    push {r14}
    bl tlsr82xx_timer0_irq_tick
    pop {r0}
    mov lr, r0
    bx lr
