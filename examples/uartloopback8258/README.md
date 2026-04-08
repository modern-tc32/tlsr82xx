# uartloopback8258

UART loopback self-test for TLSR8258/TB03F.

## Wiring

- `TX` = `PB1`
- `RX` = `PA0`
- Connect `PB1` to `PA0` for loopback.
- Common GND is required.

## Behavior

- Sends one byte (`'U'`) every second.
- Waits up to 50 ms for echo.
- Echo OK: white LED toggles, yellow LED off.
- Echo timeout/mismatch: yellow LED toggles, white LED off.

## Build

```sh
make debug
make release
```
