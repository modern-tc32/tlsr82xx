# pm_short8258

Power-management smoke test for TLSR8258 using the Rust short-sleep path.

## Behavior

- 32k source: `InternalRc`
- Wake source: `TIMER`
- Sleep API: `pm::Pm::init(pm::Config::internal_rc()).sleep_ms_short(...)`
- Four-step cycle:
  - `DeepSleepRetentionLow8K`
  - `DeepSleepRetentionLow16K`
  - `DeepSleepRetentionLow32K`
  - `DeepSleep`

Before each sleep, the white LED blinks the current step number:

- `1` on initial start
- `2` after the first retention wake
- `3` after the second retention wake
- `4` before deep sleep

After `DeepSleep`, RAM is reset and the sequence restarts from `1`.

## Build And Flash

```bash
make -C tlsr82xx/examples/pm_short8258 release
python3 TlsrPgm.py --tcp 192.168.70.44:55555 -a 100 -s -m we 0 tlsr82xx/target/tc32-unknown-none-elf/release/pm_short8258.bin
```
