.PHONY: all help debug release rgb-debug rgb-release uart-debug uart-release bin bin-release clean check-env

TARGET := tc32-unknown-none-elf
PROFILE ?= debug
TC32_TOOLCHAIN ?= $(abspath ../../../toolchains/tc32-stage1)
TC32_CARGO ?= $(TC32_TOOLCHAIN)/bin/cargo
TC32_RUSTC ?= $(TC32_TOOLCHAIN)/bin/rustc
TC32_LLVM_BIN ?= $(TC32_TOOLCHAIN)/llvm/bin
TARGET_DIR := ../../target/$(TARGET)
ELF_debug := $(TARGET_DIR)/debug/tlsr82xx-blink8258
ELF_release := $(TARGET_DIR)/release/tlsr82xx-blink8258
BIN_debug := $(ELF_debug).bin
BIN_release := $(ELF_release).bin
RGB_ELF_debug := $(TARGET_DIR)/debug/rgb8258
RGB_ELF_release := $(TARGET_DIR)/release/rgb8258
RGB_BIN_debug := $(RGB_ELF_debug).bin
RGB_BIN_release := $(RGB_ELF_release).bin
UART_ELF_debug := $(TARGET_DIR)/debug/uart8258
UART_ELF_release := $(TARGET_DIR)/release/uart8258
UART_BIN_debug := $(UART_ELF_debug).bin
UART_BIN_release := $(UART_ELF_release).bin

all: debug

help:
	@echo "Targets:"
	@echo "  make debug        Build debug ELF and tlsr82xx-blink8258.bin"
	@echo "  make release      Build release ELF and tlsr82xx-blink8258.bin"
	@echo "  make rgb-debug    Build debug ELF and rgb8258.bin"
	@echo "  make rgb-release  Build release ELF and rgb8258.bin"
	@echo "  make uart-debug   Build debug ELF and uart8258.bin"
	@echo "  make uart-release Build release ELF and uart8258.bin"
	@echo "  make bin          Alias for debug"
	@echo "  make bin-release  Alias for release"
	@echo "  make clean        Remove example build outputs"
	@echo ""
	@echo "Primary flow:"
	@echo "  1. ../../../scripts/package-stage1-tc32-toolchain.sh"
	@echo "  2. make debug"
	@echo ""
	@echo "Optional overrides:"
	@echo "  TC32_TOOLCHAIN (default: ../../../toolchains/tc32-stage1)"
	@echo "  TC32_CARGO (default: $(TC32_TOOLCHAIN)/bin/cargo)"
	@echo "  TC32_RUSTC (default: $(TC32_TOOLCHAIN)/bin/rustc)"
	@echo "  TC32_LLVM_BIN (default: $(TC32_TOOLCHAIN)/llvm/bin)"
	@echo "  TC32_SDK_DIR"
	@echo "  TC32_CLANG_COMPAT_DIR"

check-env:
	@test -x "$(TC32_CARGO)" || { echo "TC32_CARGO is not executable: $(TC32_CARGO)"; exit 1; }
	@test -x "$(TC32_RUSTC)" || { echo "TC32_RUSTC is not executable: $(TC32_RUSTC)"; exit 1; }
	@test -x "$(TC32_LLVM_BIN)/clang" || { echo "clang is not executable: $(TC32_LLVM_BIN)/clang"; exit 1; }
	@test -x "$(TC32_LLVM_BIN)/ld.lld" || { echo "ld.lld is not executable: $(TC32_LLVM_BIN)/ld.lld"; exit 1; }
	@test -x "$(TC32_LLVM_BIN)/llvm-objcopy" || { echo "llvm-objcopy is not executable: $(TC32_LLVM_BIN)/llvm-objcopy"; exit 1; }

debug: $(BIN_debug)
	@echo "ELF: $(ELF_debug)"
	@echo "BIN: $(BIN_debug)"

release: $(BIN_release)
	@echo "ELF: $(ELF_release)"
	@echo "BIN: $(BIN_release)"

rgb-debug: $(RGB_BIN_debug)
	@echo "ELF: $(RGB_ELF_debug)"
	@echo "BIN: $(RGB_BIN_debug)"

rgb-release: $(RGB_BIN_release)
	@echo "ELF: $(RGB_ELF_release)"
	@echo "BIN: $(RGB_BIN_release)"

uart-debug: $(UART_BIN_debug)
	@echo "ELF: $(UART_ELF_debug)"
	@echo "BIN: $(UART_BIN_debug)"

uart-release: $(UART_BIN_release)
	@echo "ELF: $(UART_ELF_release)"
	@echo "BIN: $(UART_BIN_release)"

bin: debug

bin-release: release

$(ELF_debug): check-env
	RUSTC="$(TC32_RUSTC)" \
	TC32_LLVM_BIN="$(TC32_LLVM_BIN)" \
	CARGO_TARGET_TC32_UNKNOWN_NONE_ELF_LINKER="$(TC32_LLVM_BIN)/ld.lld" \
	"$(TC32_CARGO)" build

$(ELF_release): check-env
	RUSTC="$(TC32_RUSTC)" \
	TC32_LLVM_BIN="$(TC32_LLVM_BIN)" \
	CARGO_TARGET_TC32_UNKNOWN_NONE_ELF_LINKER="$(TC32_LLVM_BIN)/ld.lld" \
	"$(TC32_CARGO)" build --release

$(RGB_ELF_debug): check-env
	RUSTC="$(TC32_RUSTC)" \
	TC32_LLVM_BIN="$(TC32_LLVM_BIN)" \
	CARGO_TARGET_TC32_UNKNOWN_NONE_ELF_LINKER="$(TC32_LLVM_BIN)/ld.lld" \
	"$(TC32_CARGO)" build --bin rgb8258

$(RGB_ELF_release): check-env
	RUSTC="$(TC32_RUSTC)" \
	TC32_LLVM_BIN="$(TC32_LLVM_BIN)" \
	CARGO_TARGET_TC32_UNKNOWN_NONE_ELF_LINKER="$(TC32_LLVM_BIN)/ld.lld" \
	"$(TC32_CARGO)" build --release --bin rgb8258

$(UART_ELF_debug): check-env
	RUSTC="$(TC32_RUSTC)" \
	TC32_LLVM_BIN="$(TC32_LLVM_BIN)" \
	CARGO_TARGET_TC32_UNKNOWN_NONE_ELF_LINKER="$(TC32_LLVM_BIN)/ld.lld" \
	"$(TC32_CARGO)" build --bin uart8258

$(UART_ELF_release): check-env
	RUSTC="$(TC32_RUSTC)" \
	TC32_LLVM_BIN="$(TC32_LLVM_BIN)" \
	CARGO_TARGET_TC32_UNKNOWN_NONE_ELF_LINKER="$(TC32_LLVM_BIN)/ld.lld" \
	"$(TC32_CARGO)" build --release --bin uart8258

$(BIN_debug): $(ELF_debug)
	"$(TC32_LLVM_BIN)/llvm-objcopy" -O binary "$(ELF_debug)" "$(BIN_debug)"

$(BIN_release): $(ELF_release)
	"$(TC32_LLVM_BIN)/llvm-objcopy" -O binary "$(ELF_release)" "$(BIN_release)"

$(RGB_BIN_debug): $(RGB_ELF_debug)
	"$(TC32_LLVM_BIN)/llvm-objcopy" -O binary "$(RGB_ELF_debug)" "$(RGB_BIN_debug)"

$(RGB_BIN_release): $(RGB_ELF_release)
	"$(TC32_LLVM_BIN)/llvm-objcopy" -O binary "$(RGB_ELF_release)" "$(RGB_BIN_release)"

$(UART_BIN_debug): $(UART_ELF_debug)
	"$(TC32_LLVM_BIN)/llvm-objcopy" -O binary "$(UART_ELF_debug)" "$(UART_BIN_debug)"

$(UART_BIN_release): $(UART_ELF_release)
	"$(TC32_LLVM_BIN)/llvm-objcopy" -O binary "$(UART_ELF_release)" "$(UART_BIN_release)"

clean:
	"$(TC32_CARGO)" clean
