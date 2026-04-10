# `ble_ota_boot8258`

Minimal TLSR8258 bootloader example intended for BLE OTA flow.

Layout (512KB flash):

- bootloader: `0x00000..0x07FFF` (32KB)
- app slot: `0x08000..0x7EFFF`
- metadata: `0x7F000..0x7FFFF`

Boot flow:

- on reset, reads metadata at `0x7F000`
- if metadata is valid, jumps to app entry `0x8000`
- otherwise stays in OTA advertising mode

Build:

```sh
cd tlsr82xx/examples/ble_ota_boot8258
make release
```

Flash bootloader:

```sh
python3 TlsrPgm.py -p /dev/cu.usbserial-10 -m -s -a 100 we 0 ../../target/tc32-unknown-none-elf/release/tlsr82xx-ble-ota-boot8258.bin
```

See uploader helper:

- `../tools/ota_upload_telink.py`

RAM telemetry:

```sh
nm -n ../../target/tc32-unknown-none-elf/release/tlsr82xx-ble-ota-boot8258 | grep BLE_OTA_BOOT_STATUS
python3 TlsrPgm.py -p /dev/cu.usbserial-10 -r -s -a 200 ds <ADDRESS> 0x30
```
