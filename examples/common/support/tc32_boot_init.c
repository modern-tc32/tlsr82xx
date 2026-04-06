#include <stdint.h>

extern int main(void);

extern uint32_t _dstored_;
extern uint32_t _start_data_;
extern uint32_t _end_data_;
extern uint32_t _start_bss_;
extern uint32_t _end_bss_;
extern uint32_t _custom_stored_;
extern uint32_t _start_custom_data_;
extern uint32_t _end_custom_data_;
extern uint32_t _start_custom_bss_;
extern uint32_t _end_custom_bss_;
extern uint32_t _stack_end_;
extern uint32_t _ictag_start_;
extern uint32_t _ictag_end_;
extern uint32_t _ramcode_size_align_256_;
extern unsigned char tl_multi_addr __attribute__((weak));

static inline volatile uint8_t *mmio8(uintptr_t addr) {
    return (volatile uint8_t *)addr;
}

static inline volatile uint32_t *mmio32(uintptr_t addr) {
    return (volatile uint32_t *)addr;
}

static __attribute__((always_inline)) inline uint8_t analog_read_u8(uint8_t reg) {
    volatile uint8_t *const ana = mmio8(0x8000b8);
    ana[0] = reg;
    ana[2] = 0x40;
    while ((ana[2] & 1u) != 0) {
    }
    return ana[1];
}

static __attribute__((always_inline)) inline void flash_wakeup(void) {
    volatile uint8_t *const flash = mmio8(0x80000c);

    flash[1] = 0;
    flash[0] = 0xab;
    for (volatile unsigned int i = 0; i <= 6; ++i) {
    }
    flash[1] = 1;
}

static __attribute__((always_inline)) inline void efuse_delay(void) {
    for (volatile unsigned int i = 0; i < 110; ++i) {
    }
}

static __attribute__((always_inline)) inline void init_icache(void) {
    uint32_t *tag = &_ictag_start_;
    while (tag < &_ictag_end_) {
        *tag++ = 0;
    }

    volatile uint8_t *const cache = mmio8(0x80060c);
    uint8_t lines = (uint8_t)(((uintptr_t)&_ramcode_size_align_256_) >> 8);
    cache[0] = lines;
    cache[1] = (uint8_t)(lines + 1u);
}

static __attribute__((always_inline)) inline void system_on_for_flash(void) {
    *mmio32(0x800060) = 0xff080000u;
    *mmio8(0x800064) = 0xffu;
    *mmio8(0x800065) = 0xf7u;
}

static __attribute__((always_inline)) inline void fill_stack_pattern(void) {
    uint32_t *p = &_end_custom_bss_;
    uint32_t *const end = &_stack_end_;
    while (p < end) {
        *p++ = 0xffffffffu;
    }
}

static __attribute__((always_inline)) inline void copy_words(
    uint32_t *dst,
    uint32_t *end,
    const uint32_t *src
) {
    while (dst < end) {
        *dst++ = *src++;
    }
}

static __attribute__((always_inline)) inline void zero_words(uint32_t *dst, uint32_t *end) {
    while (dst < end) {
        *dst++ = 0;
    }
}

__attribute__((noreturn, section(".vectors.boot"))) void __tc32_boot_init(void) {
    uint8_t wake_flag;

    init_icache();
    system_on_for_flash();
    flash_wakeup();
    efuse_delay();

    wake_flag = analog_read_u8(0x7e);
    if ((wake_flag & 1u) != 0) {
        *mmio8(0x80063e) = (uint8_t)tl_multi_addr;
    } else {
        fill_stack_pattern();
        copy_words(&_start_data_, &_end_data_, &_dstored_);
        zero_words(&_start_bss_, &_end_bss_);
        copy_words(&_start_custom_data_, &_end_custom_data_, &_custom_stored_);
        zero_words(&_start_custom_bss_, &_end_custom_bss_);
    }

    (void)main();
    for (;;) {
    }
}
