    .syntax unified
    .thumb

    .section .ram_code, "ax", %progbits
    .align 2
    .globl __irq
    .thumb_func
    .extern irq_handler

__irq:
    push {r14}
    push {r0-r7}
    .short 0x6bd8

    mov r1, r8
    mov r2, r9
    mov r3, r10
    mov r4, r11
    mov r5, r12
    push {r0-r5}

    bl irq_handler

    pop {r0-r5}
    mov r8, r1
    mov r9, r2
    mov r10, r3
    mov r11, r4
    mov r12, r5

    .short 0x6bd0
    pop {r0-r7}
    .short 0x6900
