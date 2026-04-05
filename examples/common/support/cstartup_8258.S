#include "version_cfg.h"

    .syntax unified
    .thumb

    .extern irq_handler
    .extern main
    .extern __tc32_boot_init
    .extern _bin_size_div_16
    .extern _bin_size_
    .extern _stack_end_

    .section .vectors, "ax", %progbits
    .globl __start
    .globl __reset
    .globl __irq
    .globl start_suspend
    .thumb_func
__start:
    b __reset
    .word FILE_VERSION
    .org 0x8
    .word 0x544c4e4b
    .word (0x00880000 + _bin_size_div_16)
    .org 0x10
    b __irq
    .org 0x12
    .short MANUFACTURER_CODE_TELINK
    .short IMAGE_TYPE
    .org 0x18
    .word _bin_size_
    .org 0x20

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

    .align 2
    .thumb_func
__reset:
    ldr r0, =_stack_end_
    mov sp, r0
    bl __tc32_boot_init
.Lhang:
    b .Lhang

    .align 2
    .thumb_func
__irq:
    push {lr}
    bl irq_handler
    pop {pc}

    .text
