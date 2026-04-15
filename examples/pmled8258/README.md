# pmled8258

Power-management diagnostic example for TLSR8258 in Rust.

The firmware cycles through PM test cases and shows startup/wakeup information via LEDs after each wake.

## Current Test Matrix

- API: `pm::long_sleep_32k(...)`
- Wake source: `TIMER`
- Sleep duration: `2s`
- Cases:
  - `RetentionLow8K` with `RC32K`
  - `RetentionLow8K` with `XTAL32K`
  - `RetentionLow16K` with `RC32K`
  - `RetentionLow16K` with `XTAL32K`
  - `RetentionLow32K` with `RC32K`
  - `RetentionLow32K` with `XTAL32K`
  - `DeepSleep` with `RC32K`
  - `DeepSleep` with `XTAL32K`

## LED Protocol

On first RAM initialization:
- White+Yellow ON for `3s` (cycle-start marker).

Then each active window shows `X-F-Y-S`:
- `X` (white, long): wake class
  - `1` = cold boot
  - `2` = deep wake
  - `3/4/5` = deep-retention wake (8K/16K/32K)
- `F` (white, short): startup wakeup-flag bucket (`PM_STARTUP_DBG_WAKEUP_FLAG`)
  - `1` for raw `0`
  - `2` for raw `1`
  - `3` for all other values
- `Y` (yellow, long): previous 32k source
  - `1` = RC32K
  - `2` = XTAL32K
- `S` (yellow, short): previous test step index (`1..8`)
  - Uses doubled OFF gap for easier counting.

## Important Behavior

- Steps `1..6` (retention modes) preserve RAM, so step index advances.
- Steps `7..8` (`DeepSleep`) do not preserve RAM.
- After `DeepSleep`, RAM state resets and the sequence restarts from the first-start marker.
- This is expected and indicates cold-boot restart semantics for deep sleep without retention.

## Build And Flash (stage2)

```bash
make -C tlsr82xx/examples/pmled8258 release
python3 TlsrPgm.py --tcp 192.168.70.44:55555 -a 100 -s -m we 0 tlsr82xx/target/tc32-unknown-none-elf/release/pmled8258.bin
```
