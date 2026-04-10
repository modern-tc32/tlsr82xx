#include <stdarg.h>
#include <stdint.h>

void tlkapi_debug_init(void) {}

int tlk_printf(const char *format, ...) {
    (void)format;
    return 0;
}

void tlkapi_send_str_data(char *str, uint8_t *data, uint32_t data_len) {
    (void)str;
    (void)data;
    (void)data_len;
}
