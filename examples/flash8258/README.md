# `flash8258`

Minimal flash inspection example for TLSR8258 using `tlsr82xx-hal::flash`.

What it does:

- initializes the board and HAL startup path
- reads flash MID, UID and VDD_F calibration byte
- cycles LED states to show that reads succeeded

Current meaning:

- frame 1: vendor class
- `LED_Y` on means Zbit flash
- `LED_W` on means non-Zbit flash
- frame 2: UID read result
- `LED_Y` on means `read_uid_default()` succeeded
- `LED_W` mirrors bit 0 of `uid[0]`
- frame 3: calibration byte
- `LED_Y` on means calibration byte is present (`!= 0xff`)
- `LED_W` mirrors bit 0 of the calibration byte

Build:

```sh
cd tlsr82xx/examples/flash8258
make release
```
