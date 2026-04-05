# `blink8258`

Minimal bare-metal Rust blink example for TLSR8258 on `tc32-unknown-none-elf`.

Current assumptions:

- chip: `MCU_CORE_8258`
- board config: `BOARD_8258_TB03F`
- clock: `48 MHz`
- blink output: `LED_Y` on `PB4`
- `LED_W` on `PB5` mirrors the opposite state

The example uses:

- `tlsr82xx-hal` GPIO on top of the generated PAC
- `build.rs` to compile the required Telink SDK and startup support objects with `clang --target=tc32`
- `ld.lld` from the packaged TC32-enabled toolchain for final linking
- local startup/linker/support files under this example directory, so it no longer depends on `examples-rust/blink_tb03f`

Additional binary:

- `rgb8258` drives `LED_R`/`LED_G`/`LED_B` on `PC2`/`PC3`/`PC4` with hardware PWM

## Prepare toolchain

From the repo root:

```sh
./scripts/package-stage1-tc32-toolchain.sh
```

This creates:

```sh
toolchains/tc32-stage1-aarch64-apple-darwin
toolchains/tc32-stage1
```

`toolchains/tc32-stage1` is the default path used by the local `Makefile`.

Optional overrides if your SDK layout differs:

```sh
export TC32_SDK_DIR=/abs/path/to/tl_zigbee_sdk
export TC32_CLANG_COMPAT_DIR=/abs/path/to/clang_compat
```

## Build

From `tlsr82xx/examples/blink8258`:

```sh
make debug
```

Release build:

```sh
make release
```

Build the RGB PWM binary directly:

```sh
make rgb-release
```

Outputs:

```sh
../../target/tc32-unknown-none-elf/debug/tlsr82xx-blink8258
../../target/tc32-unknown-none-elf/debug/tlsr82xx-blink8258.bin
../../target/tc32-unknown-none-elf/release/tlsr82xx-blink8258
../../target/tc32-unknown-none-elf/release/tlsr82xx-blink8258.bin
```

Optional disassembly check:

```sh
../../../toolchains/tc32-stage1/llvm/bin/llvm-objdump -d \
  ../../target/tc32-unknown-none-elf/release/tlsr82xx-blink8258 | head
```
