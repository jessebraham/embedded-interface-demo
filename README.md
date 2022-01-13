# Embedded Interface Demo

![MIT/Apache-2.0 licensed](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue?style=flat-square)

A simple demonstration of a self-hosted web interface running on an ESP32-C3, in Rust. Starts a soft access point and a web server running on-device, and serves the bundled single-page app.

This example is intended to be run on the [ESP32-C3-DevKitM-1] development board, however should work with any board based on the ESP32-C3 with some minor modifications.

[esp32-c3-devkitm-1]: https://docs.espressif.com/projects/esp-idf/en/latest/esp32c3/hw-reference/esp32c3/user-guide-devkitm-1.html

## Quickstart

A number of tools are required in order to build and flash this project. Please ensure that each dependency listed below has been properly installed on your system.

| Tool             | Download                           |
| ---------------- | ---------------------------------- |
| `cargo`          | https://rustup.rs/                 |
| `npm`            | https://nodejs.org/en/             |
| `cargo espflash` | https://github.com/esp-rs/espflash |

Commands for building and flashing the firmware have been included following the workflow defined by [cargo-xtask]. Please see the _cargo-xtask_ README for more information.

[cargo-xtask]: https://github.com/matklad/cargo-xtask

### Build the Interface and Firmware

In order to build the web interface, place the bundled single-page application in the correct location, and subsequently build the firmware, from the root of the repository run:

```shell
$ cargo xtask build
```

### Flash the Firmware

With your development board plugged in to your computer via a USB cable, you can then flash the firmware to the board and open a serial monitor when the process has completed:

```shell
$ cargo xtask flash
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
