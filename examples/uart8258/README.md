# `uart8258`

Bare-metal Rust UART example for TLSR8258 on `tc32-unknown-none-elf`.

- board config: `BOARD_8258_TB03F`
- clock: `48 MHz`
- UART TX: `PB1`
- UART RX: `PA0`

Build from `tlsr82xx/examples/uart8258`:

```sh
make debug
make release
```
