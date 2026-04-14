# pmled8258

Power-management example for TLSR8258 in Rust: `sleep -> wake -> blink -> sleep`.

## Current behavior

- 32k source: internal RC.
- Sleep source: TIMER.
- Sleep duration: 2 seconds.
- Sleep modes: alternates between:
  - `DeepSleepRetentionLow16K`
  - `DeepSleepRetentionLow32K`
- LED indication:
  - short yellow pulse on each active window.

## Retention diagnostics

The example exports PM diagnostic variables in RAM:

- `PM_DIAG_MAGIC`
- `PM_DIAG_BOOT_COUNT`
- `PM_DIAG_WAKE_COUNT`
- `PM_DIAG_LOOP_COUNT`
- `PM_DIAG_WAKE_ORIGIN` (`0=ColdBoot`, `1=DeepWake`, `2=DeepRetentionWake`)
- `PM_DIAG_WAKE_SRC_RAW`
- `PM_DIAG_LAST_SLEEP_MODE`
- `PM_DIAG_NEXT_MODE`

Get their addresses from ELF:

```bash
toolchains/tc32-stage2/llvm/bin/llvm-nm -n tlsr82xx/target/tc32-unknown-none-elf/release/pmled8258 | rg PM_DIAG
```

Read RAM via SWire (example: dump 32 bytes at diagnostic base address):

```bash
python3 TlsrPgm.py --tcp 192.168.70.44:55555 -a 100 -s ds 0x<ADDR> 0x20
```

## Build and flash

Build (stage2):

```bash
make -C tlsr82xx/examples/pmled8258 release
```

Flash:

```bash
python3 TlsrPgm.py --tcp 192.168.70.44:55555 -a 100 -s -m we 0 tlsr82xx/target/tc32-unknown-none-elf/release/pmled8258.bin
```
