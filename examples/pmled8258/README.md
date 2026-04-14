# pmled8258

Power-management example for TLSR8258 in Rust: `sleep -> wake -> blink -> sleep`.

## Current behavior

Current mode: `DeepSleep + TIMER` (32k RC long sleep path).

Sequence:

1. On true cold boot (`StartupState::Boot`), LED is white briefly.
2. Device blinks yellow shortly to mark active window.
3. Device enters `DeepSleep`.
4. Timer wakes MCU, firmware starts again, and the loop repeats.

During normal deep-sleep wake cycles there is no repeated white startup pulse.

## Why it previously failed

Deep-sleep wake on 8258 is sensitive to HAL/vendor PM alignment.  
The working setup uses stage2 toolchain and keeps PM startup/wakeup flow close to `drivers/pm.s`.

## Build and flash

Build (stage2, default Rust PM path):

```bash
make -C tlsr82xx/examples/pmled8258 release
```

Build (stage2, vendor PM fallback):

```bash
cd tlsr82xx/examples/pmled8258
PATH="$(pwd)/../../../toolchains/tc32-stage2/llvm/bin:$PATH" \
RUSTC="$(pwd)/../../../toolchains/tc32-stage2/bin/rustc" \
TC32_LLVM_BIN="$(pwd)/../../../toolchains/tc32-stage2/llvm/bin" \
CARGO_TARGET_TC32_UNKNOWN_NONE_ELF_LINKER="$(pwd)/../../../toolchains/tc32-stage2/llvm/bin/ld.lld" \
"$(pwd)/../../../toolchains/tc32-stage2/bin/cargo" build --release --target tc32-unknown-none-elf --features vendor-pm
```

Flash:

```bash
python3 TlsrPgm.py --tcp 192.168.70.44:55555 -a 100 -s -m we 0 tlsr82xx/target/tc32-unknown-none-elf/release/pmled8258.bin
```
