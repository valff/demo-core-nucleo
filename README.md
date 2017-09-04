blink-stm32l4
=============

Example blinking program for [NUCLEO-L496ZG-P][nucleo-l496zg-p] board using
[Drone][drone] RTOS.

## Effects

* Smooth blinking with the all three user LEDs.
* Running MCU at the full speed (80 MHz).
* Using the on-board LSE for MSI auto-calibration.
* Printing messages through ITM.

## Usage

Flash the board with the following command:

```sh
$ cargo drone flash --release
```

Listen to the ITM stream for connected device with the following command:

```sh
$ cargo drone server --itm
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[drone]: https://github.com/valff/drone
[nucleo-l496zg-p]: http://www.st.com/en/evaluation-tools/nucleo-l496zg-p.html
