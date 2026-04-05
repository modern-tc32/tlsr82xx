# `rgb8258`

Bare-metal Rust PWM RGB example for TLSR8258 on `tc32-unknown-none-elf`.

- board config: `BOARD_8258_TB03F`
- clock: `48 MHz`
- outputs: `LED_R`/`LED_G`/`LED_B` on `PC2`/`PC3`/`PC4`

Build from `tlsr82xx/examples/rgb8258`:

```sh
make debug
make release
```
