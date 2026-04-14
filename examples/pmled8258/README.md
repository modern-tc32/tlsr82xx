# pmled8258

Power-management example for TLSR8258: `sleep -> wake -> blink -> sleep`.

## Current behavior

Current mode: `DeepSleep + TIMER`.

Sequence:

1. On cold boot (`StartupState::Boot`), LED is white for ~2 seconds.
2. Device enters `DeepSleep` for `SLEEP_MS`.
3. Timer wakeup causes MCU restart (`StartupState::Deep`).
4. Short yellow blink (`WAKE_BLINK_US`) marks wake event.
5. Device enters `DeepSleep` again.

## Why it previously failed

The issue was not the return type of `platform::init()` itself.

This is valid by itself:

```rust
pub fn init() -> tlsr82xx_hal::startup::StartupState {
    tlsr82xx_hal::startup::init()
}
```

The real problem was divergence from the known-good `blink8258` boot path while PM experiments were added around startup. That caused unstable early boot (hangs / pseudo boot loops).

What stabilized the example:

1. Bring `platform::init()` back close to the proven path (`startup::init()` without extra logic).
2. Re-introduce PM behavior only after stable boot is confirmed.
3. Fix timer PM path issues in `startup`/PM flow (register offset / mode bits), which were causing lockups and pseudo boot loops.

Bottom line: stabilize boot first, then add sleep path.

## Build and flash

Build (stage2):

```bash
make -C tlsr82xx/examples/pmled8258 release
```

Flash:

```bash
python3 TlsrPgm.py --tcp 192.168.70.44:55555 -a 100 -s -m we 0 tlsr82xx/target/tc32-unknown-none-elf/release/pmled8258.bin
```
