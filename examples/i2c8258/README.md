# `i2c8258`

Minimal TLSR8258 I2C scan example using `tlsr82xx-hal::i2c` with the local `embedded-hal` API.

What it does:

- initializes the standard 8258 startup path
- configures hardware I2C on `I2cPinGroup::C0C1`
- scans 7-bit addresses `0x08..=0x77` by sending an address phase and checking ACK
- updates LEDs every 500 ms

Current LED meaning:

- `LED_Y` on means at least one device ACKed on the bus
- `LED_W` mirrors bit 0 of the first ACKed address

Notes:

- this example assumes external pull-ups on SDA/SCL
- default pin group is `C0/C1`; change `I2cPinGroup::C0C1` in `src/main.rs` if your board uses another pair
- default speed is `100 kHz`

Build:

```sh
cd tlsr82xx/examples/i2c8258
make release
```
