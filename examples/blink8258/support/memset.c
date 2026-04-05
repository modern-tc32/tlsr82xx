#include "common/types.h"

void *memset(void *dest, int value, size_t len) {
    u8 *ptr = (u8 *)dest;
    u8 byte = (u8)value;

    while (len--) {
        *ptr++ = byte;
    }

    return dest;
}
