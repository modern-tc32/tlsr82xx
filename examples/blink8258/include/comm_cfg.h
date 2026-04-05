#pragma once

#define BOOT_LOADER_MODE                0
#define BOOT_LOADER_IMAGE_ADDR          0x0

#if (BOOT_LOADER_MODE)
#define APP_IMAGE_ADDR                  0x8000
#else
#define APP_IMAGE_ADDR                  0x0
#endif

#define TLSR_8267                       0x00
#define TLSR_8269                       0x01
#define TLSR_8258_512K                  0x02
#define TLSR_8258_1M                    0x03
#define TLSR_8278                       0x04
#define TLSR_B91                        0x05
#define TLSR_B92                        0x06
#define TLSR_TL721X                     0x07
#define TLSR_TL321X                     0x08

#if (BOOT_LOADER_MODE)
#define IMAGE_TYPE_BOOT_FLAG            1
#else
#define IMAGE_TYPE_BOOT_FLAG            0
#endif

#define IMAGE_TYPE_BOOTLOADER           0xFF
#define IMAGE_TYPE_GW                   (0x00 | (IMAGE_TYPE_BOOT_FLAG << 7))
#define IMAGE_TYPE_LIGHT                (0x01 | (IMAGE_TYPE_BOOT_FLAG << 7))
#define IMAGE_TYPE_SWITCH               (0x02 | (IMAGE_TYPE_BOOT_FLAG << 7))
#define IMAGE_TYPE_CONTACT_SENSOR       (0x03 | (IMAGE_TYPE_BOOT_FLAG << 7))
