# `spi8258`

Minimal TLSR8258 SPI bus example using `tlsr82xx-hal::spi` and `embedded-hal::spi::SpiBus<u8>`.

What it does:

- initializes the standard 8258 startup path
- configures hardware SPI master on `SpiPinGroup::A2A3A4D6`
- runs `transfer_in_place()` on a single byte every 500 ms
- mirrors bits of the received byte to LEDs

Suggested hardware check:

- connect `MOSI` to `MISO` for a loopback test
- on `A2A3A4D6` group this means `PA2 -> PA3`

Current LED meaning:

- `LED_Y` mirrors bit 0 of the received byte
- `LED_W` mirrors bit 1 of the received byte

Build:

```sh
cd tlsr82xx/examples/spi8258
make release
```
