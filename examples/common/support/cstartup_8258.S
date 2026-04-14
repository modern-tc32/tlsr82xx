#include "version_cfg.h"

    .equ IRQ_STK_SIZE, 0x200

    .syntax unified
    .thumb

    .extern __irq
    .extern main
    .extern __tc32_boot_init
    .extern _bin_size_div_16
    .extern _bin_size_
    .extern _stack_end_

    .section .vectors, "ax", %progbits
    .globl __start
    .globl __reset
    .globl start_suspend
    .thumb_func
__start:
    b __reset
    .word FILE_VERSION
    .org 0x8
    .word 0x544c4e4b
    .word (0x00880000 + _bin_size_div_16)
    .org 0x10
    b __irq_vector
    .org 0x12
    .short MANUFACTURER_CODE_TELINK
    .short IMAGE_TYPE
    .org 0x18
    .word _bin_size_
    .org 0x20

    .thumb_func
__irq_vector:
    ldr r0, =__irq
    bx r0

    .section .ram_code, "ax", %progbits
    .align 2
    .thumb_func
start_suspend:
    push {r2, r3}
    movs r2, #129
    ldr r3, =0x80006f
    strb r2, [r3]
    .rept 64
    nop
    .endr
    pop {r2, r3}
    bx lr

    .section .vectors, "ax", %progbits
    .align 2
    .thumb_func
__reset:
    ldr r0, =0x12
    .short 0x6bc0
    ldr r0, =(irq_stk + IRQ_STK_SIZE)
    mov sp, r0
    ldr r0, =0x13
    .short 0x6bc0
    ldr r0, =_stack_end_
    mov sp, r0
    bl __tc32_boot_init
.Lhang:
    b .Lhang

    .text

    .section .bss
    .align 4
    .lcomm irq_stk, IRQ_STK_SIZE
