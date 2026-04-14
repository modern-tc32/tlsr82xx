.PHONY: all help debug release bin bin-release clean check-env

TARGET := tc32-unknown-none-elf
PROFILE ?= debug
TC32_TOOLCHAIN ?= $(abspath ../../../toolchains/tc32-stage2)
TC32_CARGO ?= $(TC32_TOOLCHAIN)/bin/cargo
TC32_RUSTC ?= $(TC32_TOOLCHAIN)/bin/rustc
TC32_LLVM_BIN ?= $(TC32_TOOLCHAIN)/llvm/bin
TARGET_DIR := ../../target/$(TARGET)
ELF_debug := $(TARGET_DIR)/debug/tlsr82xx-ble-beacon8258
ELF_release := $(TARGET_DIR)/release/tlsr82xx-ble-beacon8258
BIN_debug := $(ELF_debug).bin
BIN_release := $(ELF_release).bin
all: debug

help:
	@echo "Targets:"
	@echo "  make debug        Build debug ELF and tlsr82xx-ble-beacon8258.bin"
	@echo "  make release      Build release ELF and tlsr82xx-ble-beacon8258.bin"
	@echo "  make bin          Alias for debug"
	@echo "  make bin-release  Alias for release"
	@echo "  make clean        Remove example build outputs"

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

bin: debug

bin-release: release

$(ELF_debug): check-env
	PATH="$(TC32_LLVM_BIN):$$PATH" \
	RUSTC="$(TC32_RUSTC)" \
	TC32_LLVM_BIN="$(TC32_LLVM_BIN)" \
	CARGO_TARGET_TC32_UNKNOWN_NONE_ELF_LINKER="$(TC32_LLVM_BIN)/ld.lld" \
	"$(TC32_CARGO)" build --target "$(TARGET)"

$(ELF_release): check-env
	PATH="$(TC32_LLVM_BIN):$$PATH" \
	RUSTC="$(TC32_RUSTC)" \
	TC32_LLVM_BIN="$(TC32_LLVM_BIN)" \
	CARGO_TARGET_TC32_UNKNOWN_NONE_ELF_LINKER="$(TC32_LLVM_BIN)/ld.lld" \
	"$(TC32_CARGO)" build --release --target "$(TARGET)"

$(BIN_debug): $(ELF_debug)
	"$(TC32_LLVM_BIN)/llvm-objcopy" -O binary "$(ELF_debug)" "$(BIN_debug)"

$(BIN_release): $(ELF_release)
	"$(TC32_LLVM_BIN)/llvm-objcopy" -O binary "$(ELF_release)" "$(BIN_release)"

clean:
	"$(TC32_CARGO)" clean
