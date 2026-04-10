.PHONY: all clean flash

TARGET := ble_ota_boot8258
OUT_DIR := build
OBJ_DIR := $(OUT_DIR)/obj

TOOLCHAIN ?= $(abspath ../../../tc32-vendor)
SDK ?= $(abspath ../../../../tc_ble_single_sdk/tc_ble_single_sdk)

CC := $(TOOLCHAIN)/bin/tc32-elf-gcc
LD := $(TOOLCHAIN)/bin/tc32-elf-ld
OBJCOPY := $(TOOLCHAIN)/bin/tc32-elf-objcopy

LINK_SCRIPT := $(SDK)/project/tlsr_tc32/B85/boot.link

CFLAGS := -ffunction-sections -fdata-sections -O2 -std=gnu99 \
	-fshort-wchar -fms-extensions -fpack-struct -fshort-enums \
	-finline-small-functions

CPPFLAGS := \
	-D__PROJECT_8258_BLE_SAMPLE__=1 \
	-DCHIP_TYPE=CHIP_TYPE_825x \
	-DMCU_STARTUP_8258=1 \
	-DMCU_CORE_8258=1 \
	-D__TL_LIB_8258__=1 \
	-I$(CURDIR)/vendor_ble_sample \
	-I$(SDK) \
	-I$(SDK)/vendor/common \
	-I$(SDK)/common \
	-I$(SDK)/drivers/B85 \
	-I$(SDK)/drivers/B85/driver_ext \
	-I$(SDK)/drivers/B85/flash \
	-I$(SDK)/stack/ble

LDFLAGS := --gc-sections -T $(LINK_SCRIPT) -L$(SDK)/proj_lib -Map $(OUT_DIR)/$(TARGET).map
LDLIBS := -llt_825x -llt_general_stack

LOCAL_SRCS := \
	vendor_ble_sample/main.c \
	vendor_ble_sample/app.c \
	vendor_ble_sample/app_att.c \
	vendor_ble_sample/app_ui.c

STARTUP_SRC := $(SDK)/boot/B85/cstartup_825x.S
SDK_DIRS_C := common application algorithm stack drivers/B85 vendor/common
SDK_C_SRCS := $(shell cd "$(SDK)" && find $(SDK_DIRS_C) -type f -name '*.c' | LC_ALL=C sort)
SDK_S_SRCS := $(shell cd "$(SDK)" && find common -type f \( -name '*.S' -o -name '*.s' \) | LC_ALL=C sort)
SDK_SRCS := $(STARTUP_SRC) $(addprefix $(SDK)/,$(SDK_C_SRCS)) $(addprefix $(SDK)/,$(SDK_S_SRCS))

LOCAL_OBJS := $(patsubst %.c,$(OBJ_DIR)/local/%.o,$(LOCAL_SRCS))
SDK_C_OBJS := $(patsubst $(SDK)/%.c,$(OBJ_DIR)/sdk/%.o,$(filter %.c,$(SDK_SRCS)))
SDK_S_OBJS := $(patsubst $(SDK)/%.S,$(OBJ_DIR)/sdk/%.o,$(filter %.S,$(SDK_SRCS)))
SDK_s_OBJS := $(patsubst $(SDK)/%.s,$(OBJ_DIR)/sdk/%.o,$(filter %.s,$(SDK_SRCS)))
OBJS := $(LOCAL_OBJS) $(SDK_C_OBJS) $(SDK_S_OBJS) $(SDK_s_OBJS)

ELF := $(OUT_DIR)/$(TARGET).elf
BIN := $(OUT_DIR)/$(TARGET).bin

all: $(BIN)

$(BIN): $(ELF)
	@mkdir -p $(dir $@)
	$(OBJCOPY) -O binary $< $@

$(ELF): $(OBJS)
	@mkdir -p $(dir $@)
	$(LD) $(LDFLAGS) -o $@ $(OBJS) $(LDLIBS)

$(OBJ_DIR)/local/%.o: %.c
	@mkdir -p $(dir $@)
	$(CC) $(CPPFLAGS) $(CFLAGS) -MMD -MP -c $< -o $@

$(OBJ_DIR)/sdk/%.o: $(SDK)/%.c
	@mkdir -p $(dir $@)
	$(CC) $(CPPFLAGS) $(CFLAGS) -MMD -MP -c $< -o $@

$(OBJ_DIR)/sdk/%.o: $(SDK)/%.S
	@mkdir -p $(dir $@)
	$(CC) $(CPPFLAGS) $(CFLAGS) -MMD -MP -c $< -o $@

$(OBJ_DIR)/sdk/%.o: $(SDK)/%.s
	@mkdir -p $(dir $@)
	$(CC) $(CPPFLAGS) $(CFLAGS) -MMD -MP -c $< -o $@

flash: $(BIN)
	python3 ../../../TlsrPgm.py -p /dev/cu.usbserial-10 -m -s -a 100 we 0 $(BIN)

clean:
	rm -rf $(OUT_DIR)
