# `radio8258`

Minimal RF control example for TLSR8258 using `tlsr82xx-hal::radio`.

What it does:

- initializes the board and HAL startup path
- configures a shared RF RX buffer
- alternates between `BleConfig::advertising(37)` and `ZigbeeConfig::new(11)`
- schedules a broadcast RX command after each mode switch
- uses the TB03F LEDs to show the active preset

Current meaning:

- `LED_Y` on: BLE 1M preset active
- `LED_W` on: Zigbee 250K preset active

Build:

```sh
cd tlsr82xx/examples/radio8258
make release
```
