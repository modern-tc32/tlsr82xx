# pmled8258_sleep_for_ms_vendor_probe

Diagnostic TLSR8258 PM probe for the `sleep_for_ms` path.

## Current Scope

- Clock source: `ExternalCrystal` only (XTAL 32k path).
- Sleep duration: `2s`.
- Wake source: `WakeupSource::TIMER`.
- Test loop: 4-step cycle (`31 -> 32 -> 33 -> 34`) for repeatability checks.
- Sleep mode in the current matrix: `Suspend`.

## Sleep Path In Use

For `Suspend`, this probe currently goes through `startup::cpu_stall(...)` in chunks.
This is intentional for diagnostics of timer wake stability in the `sleep_for_ms` flow.

For non-`Suspend` modes, the code path is prepared to call `startup::cpu_sleep_wakeup(...)`.

## LED Protocol (current firmware)

Per test step, before sleep:
- `3` white blinks (fixed preamble),
- then `N` yellow blinks where `N = 1..4` (current step index in the cycle).

Then device enters `2s` sleep and proceeds to the next step.

## Runtime Diagnostics In SRAM

The probe keeps debug markers/counters in RAM (`DBG_STAGE`, `DBG_LAST_RET`,
`DBG_CNT_*`, `NEXT_TEST_INDEX`, etc.) so state can be checked with `TlsrPgm ds`.

## Build And Flash (stage2)

```bash
make -C tlsr82xx/examples/pmled8258_sleep_for_ms_vendor_probe
python3 TlsrPgm.py --tcp 192.168.70.44:55555 -a 100 -s -m we 0 tlsr82xx/target/tc32-unknown-none-elf/debug/pmled8258_sleep_for_ms_vendor_probe.bin
```
