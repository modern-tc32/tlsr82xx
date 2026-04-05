#pragma once

#if defined(__cplusplus)
extern "C" {
#endif

#define UART_PRINTF_MODE        0
#define USB_PRINTF_MODE         0
#define ZBHCI_UART              0
#define VOLTAGE_DETECT_ENABLE   0
#define FLASH_PROTECT_ENABLE    0
#define MODULE_WATCHDOG_ENABLE  0
#define PM_ENABLE               0

#define BOARD_8258_TB03F        17

#if defined(MCU_CORE_8258)
#define BOARD                   BOARD_8258_TB03F
#define CLOCK_SYS_CLOCK_HZ      48000000
#else
#error "This example is configured for MCU_CORE_8258 only"
#endif

#include "version_cfg.h"
#include "board_8258_tb_03f.h"

typedef enum {
    EV_POLL_ED_DETECT,
    EV_POLL_HCI,
    EV_POLL_IDLE,
    EV_POLL_MAX,
} ev_poll_e;

#if defined(__cplusplus)
}
#endif
