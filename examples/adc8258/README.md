# `adc8258`

Minimal ADC example for TLSR8258 using `tlsr82xx-hal::adc`.

What it does:

- initializes the board and HAL startup path
- configures ADC GPIO input on `PB3`
- reads the current Rust single-shot ADC sample value and fluctuation
- reads the GPIO calibration Vref value
- cycles LED states to show the sampled bits

Note:

- this example uses the current Rust single-shot ADC path
- it does not yet replace the legacy startup ADC symbol path globally

Current meaning:

- frame 1: sample millivolts bit 0 on `LED_Y`, bit 1 on `LED_W`
- frame 2: fluctuation millivolts bit 0 on `LED_Y`, bit 1 on `LED_W`
- frame 3: calibration Vref bit 0 on `LED_Y`, bit 1 on `LED_W`

Build:

```sh
cd tlsr82xx/examples/adc8258
make release
```
