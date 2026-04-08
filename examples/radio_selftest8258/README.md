# `radio_selftest8258`

Self-test transmitter for TLSR8258 radio with RAM counters for programmer-based memory readout.

What it does:

- initializes platform startup and RF in `Zigbee250K` mode
- prepares a small aligned TX packet in RAM
- schedules TX (`stx2rx`) every 500 ms
- stores health counters into exported RAM struct `RADIO_SELFTEST_STATUS` (`.bss`)
- LED indication:
  - `LED_W` toggles on successful TX IRQ
  - `LED_Y` lights on TX start/timeout error

Build:

```sh
cd tlsr82xx/examples/radio_selftest8258
make release
```

Find RAM symbol address in ELF:

```sh
nm -n ../../target/tc32-unknown-none-elf/release/tlsr82xx-radio-selftest8258 | grep RADIO_SELFTEST_STATUS
```

Read RAM block with your programmer by that address (struct size is 68 bytes / `0x44`).

Fields:

- `magic` = `0x52465431` (`"RFT1"`)
- `version`
- `loops`
- `init_ok` / `init_err`
- `tx_attempts`
- `tx_start_err`
- `tx_ok`
- `tx_timeout`
- `tx_other_irq`
- `tx_wait_no_irq`
- `mode`, `phase_code`, `last_irq`, `last_rssi_dbm`
- RF/DMA snapshots: `dma3_*`, `dma_tx_rdy0`, `dma_chn_en`, `rf_mode_ctrl`, `rf_ll_ctrl*`, `rf_irq_*`

For quick RF health check, confirm that `tx_attempts` and `tx_ok` both increase after reboot.
- `last_irq`
- `last_rssi_dbm`
- `rf_mode_ctrl`
- `rf_irq_mask`
- `rf_irq_status`
