# `ble_beacon8258`

Raw BLE beacon example for TLSR8258 using `tlsr82xx-hal::radio` (no full BLE stack).

What it does:

- sends `ADV_NONCONN_IND` every ~1 second
- transmits on BLE advertising channels `37 -> 38 -> 39` for each event
- uses fixed random-static advertiser address: `C0:DE:C0:DE:82:58`
- puts payload bytes `01 02 03 04 05 06` into Manufacturer Specific Data (`AD type 0xFF`)
- exports RAM telemetry struct `BLE_BEACON_STATUS` for debug via `TlsrPgm.py`

Advertising payload:

- `AdvData = 07 FF 01 02 03 04 05 06`

Build:

```sh
cd tlsr82xx/examples/ble_beacon8258
make release
```

Flash:

```sh
python3 TlsrPgm.py -p /dev/cu.usbserial-10 -m -s -a 100 we 0 ../../target/tc32-unknown-none-elf/release/tlsr82xx-ble-beacon8258.bin
```

Check on phone:

- open `nRF Connect` or `LightBlue`
- find device with MAC `C0:DE:C0:DE:82:58`
- open advertisement details and verify Manufacturer Data contains `01 02 03 04 05 06`

RAM telemetry:

```sh
# find status symbol address
nm -n ../../target/tc32-unknown-none-elf/release/tlsr82xx-ble-beacon8258 | grep BLE_BEACON_STATUS

# read status block (0x48 bytes)
python3 TlsrPgm.py -p /dev/cu.usbserial-10 -r -s -a 200 ds <ADDRESS> 0x48
```

Telemetry highlights:

- `event_ok` / `event_fail`
- `tx_attempts` / `tx_ok` / `tx_timeout`
- `last_error`, `phase`, `last_irq`
- register snapshots: `irq_mask`, `rf_irq_status`, `rf_mode_ctrl`, `dma3_addr`, `dma3_hi`, `dma_tx_rdy`, `dma_chn_en`
