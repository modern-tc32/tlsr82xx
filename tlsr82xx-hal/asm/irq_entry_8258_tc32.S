    .section .ram_code, "ax", %progbits
    .align 2
    .globl __irq
    .code 16
    .thumb_func
    .extern irq_handler

__irq:
    tpush {r14}
    tpush {r0-r7}
    tmrss r0

    tmov r1, r8
    tmov r2, r9
    tmov r3, r10
    tmov r4, r11
    tmov r5, r12
    tpush {r0-r5}

    tjl irq_handler

    tpop {r0-r5}
    tmov r8, r1
    tmov r9, r2
    tmov r10, r3
    tmov r11, r4
    tmov r12, r5

    tmssr r0
    tpop {r0-r7}
    treti {r15}
