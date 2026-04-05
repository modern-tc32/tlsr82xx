#pragma once

#if defined(__cplusplus)
extern "C" {
#endif

#define COLOR_RGB_SUPPORT                   1
#define COLOR_CCT_SUPPORT                   0
#define BRIGHTNESS_SUPPORT                  0

#if COLOR_RGB_SUPPORT && COLOR_CCT_SUPPORT
#error "Not Support"
#elif COLOR_RGB_SUPPORT
#define COLOR_X_Y_DISABLE                   1
#endif

#define BUTTON1                             GPIO_SWS
#define PA7_FUNC                            AS_GPIO
#define PA7_OUTPUT_ENABLE                   0
#define PA7_INPUT_ENABLE                    1
#define PULL_WAKEUP_SRC_PA7                 PM_PIN_PULLUP_1M

#define BUTTON2                             GPIO_PD2
#define PD2_FUNC                            AS_GPIO
#define PD2_OUTPUT_ENABLE                   0
#define PD2_INPUT_ENABLE                    1
#define PULL_WAKEUP_SRC_PD2                 PM_PIN_PULLUP_10K

#define LED_R                               GPIO_PC2
#define LED_G                               GPIO_PC3
#define LED_B                               GPIO_PC4

#define PWM_R_CHANNEL                       0
#define PWM_R_CHANNEL_SET()                 do { } while (0)

#define PWM_G_CHANNEL                       1
#define PWM_G_CHANNEL_SET()                 do { } while (0)

#define PWM_B_CHANNEL                       2
#define PWM_B_CHANNEL_SET()                 do { } while (0)

#define R_LIGHT_PWM_CHANNEL                 PWM_R_CHANNEL
#define G_LIGHT_PWM_CHANNEL                 PWM_G_CHANNEL
#define B_LIGHT_PWM_CHANNEL                 PWM_B_CHANNEL
#define R_LIGHT_PWM_SET()                   PWM_R_CHANNEL_SET()
#define G_LIGHT_PWM_SET()                   PWM_G_CHANNEL_SET()
#define B_LIGHT_PWM_SET()                   PWM_B_CHANNEL_SET()

#define LED_Y                               GPIO_PB4
#define LED_W                               GPIO_PB5

#define PB5_FUNC                            AS_GPIO
#define PB5_OUTPUT_ENABLE                   1
#define PB5_INPUT_ENABLE                    0

#define PB4_FUNC                            AS_GPIO
#define PB4_OUTPUT_ENABLE                   1
#define PB4_INPUT_ENABLE                    0

#define LED_POWER                           LED_W
#define LED_PERMIT                          LED_Y

#define VOLTAGE_DETECT_ADC_PIN              GPIO_PC5

#if ZBHCI_UART
#define UART_TX_PIN                         UART_TX_PB1
#define UART_RX_PIN                         UART_RX_PA0
#define UART_PIN_CFG()                      do { } while (0)
#endif

#if UART_PRINTF_MODE
#define DEBUG_INFO_TX_PIN                   GPIO_PC7
#endif

#if ZBHCI_USB_PRINT || ZBHCI_USB_CDC || ZBHCI_USB_HID
#define HW_USB_CFG()                        do { usb_set_pin_en(); } while (0)
#endif

enum {
    VK_SW1 = 0x01,
    VK_SW2 = 0x02,
};

#define KB_MAP_NORMAL { {VK_SW1,}, }
#define KB_MAP_NUM    KB_MAP_NORMAL
#define KB_MAP_FN     KB_MAP_NORMAL

#define KB_DRIVE_PINS {NULL}
#define KB_SCAN_PINS  {BUTTON1}

#if defined(__cplusplus)
}
#endif
