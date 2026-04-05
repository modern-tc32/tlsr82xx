#pragma once

#include "comm_cfg.h"

#define CHIP_TYPE               TLSR_8258_512K

#define APP_RELEASE             0x10
#define APP_BUILD               0x01
#define STACK_RELEASE           0x30
#define STACK_BUILD             0x01

#define MANUFACTURER_CODE_TELINK 0x1141
#define IMAGE_TYPE              ((CHIP_TYPE << 8) | IMAGE_TYPE_LIGHT)
#define FILE_VERSION            ((APP_RELEASE << 24) | (APP_BUILD << 16) | (STACK_RELEASE << 8) | STACK_BUILD)

#define IS_BOOT_LOADER_IMAGE    0
#define RESV_FOR_APP_RAM_CODE_SIZE 0
#define IMAGE_OFFSET            APP_IMAGE_ADDR
