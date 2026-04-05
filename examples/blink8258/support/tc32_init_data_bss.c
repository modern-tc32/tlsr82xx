#include <stdint.h>

extern uint32_t _start_data_;
extern uint32_t _end_data_;
extern uint32_t _dstored_bin_;
extern uint32_t _start_bss_;
extern uint32_t _end_bss_;

void __tc32_init_data_bss(void) {
    uint32_t *dst = &_start_data_;
    uint32_t *src = &_dstored_bin_;

    while (dst < &_end_data_) {
        *dst++ = *src++;
    }

    for (dst = &_start_bss_; dst < &_end_bss_;) {
        *dst++ = 0;
    }
}
