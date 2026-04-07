    .syntax unified
    .thumb

    .section .ram_code, "ax", %progbits
    .align 2
    .globl irq_handler
    .thumb_func
irq_handler:
    ldr r0, =0x00800648
    movs r1, #1
    str r1, [r0]

    ldr r0, =0x00800623
    strb r1, [r0]

    ldr r0, =0x00800630
    movs r1, #0
    str r1, [r0]

    ldr r0, =0x00f42400
    ldr r1, =0x00800624
    str r0, [r1]

    ldr r0, =0x00800620
    ldr r1, [r0]
    movs r2, #1
    orrs r1, r2
    str r1, [r0]

    ldr r0, =stimerirq8258_irq_phase
    push {r14}
    bl stimerirq8258_rust_handler
    pop {r0}
    mov lr, r0
    bx lr

    .section .bss
    .align 1
    .globl stimerirq8258_irq_phase
stimerirq8258_irq_phase:
    .byte 0
