# Hermes-Five

[![License](https://img.shields.io/github/license/dclause/hermes-five?color=success)](/LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/dclause/hermes-five/build.yml?branch=develop&label=Build)](https://github.com/dclause/hermes-five/actions/workflows/build.yml?query=branch%3Adevelop)
[![Test Status](https://img.shields.io/github/actions/workflow/status/dclause/hermes-five/test.yml?branch=develop&label=Test)](https://github.com/dclause/hermes-five/actions/workflows/test.yml?query=branch%3Adevelop)
[![Docs Status](https://img.shields.io/docsrs/hermes-five?label=Doc)](https://docs.rs/hermes-five)
[![Code Coverage](https://codecov.io/gh/dclause/hermes-five/branch/develop/graph/badge.svg?token=KF8EFDUQ7A)](https://codecov.io/gh/dclause/hermes-five/branch/develop)
[![crates.io](https://img.shields.io/crates/v/hermes-five.svg)](https://crates.io/crates/hermes-five)
[![Documentation](https://img.shields.io/badge/Documentation-available%20here-success)](https://dclause.github.io/hermes-five/)

### The Rust Robotics & IoT Framework

**Drive and orchestrate Arduinos, ESPs, nodeMCU, RaspberryPis and all kind
of [Firmata-compatible](https://github.com/firmata) hardware in pure async Rust.
Control LEDs, sensors, motors from your laptop with the safety and speed of Rust.**

_Program robots and embedded devices with confidence. Hermes-Five gives you high-level APIs to remotely control boards (
Arduino, ESP, nodeMCU, RaspberryPI, ..), extenders (PCA9685, PCF8575, ..) LEDs, sensors, servos and more, all from safe
and asynchronous Rust code. Think _Johnny-Five_, but safer, faster, and fully async._

<img align="center" style="margin:20px 0;" alt="Schema sample of blinking led using Arduino UNO" src="/docs/public/schemas/overall.png?raw=true" />

## Documentation

Hermes-Five offers three main documentation sources:

- The [user documentation](https://dclause.github.io/hermes-five) for tutorials and concepts.
- The [API documentation](https://docs.rs/hermes-five/latest) for developer reference.
- The [examples](https://github.com/dclause/hermes-five/tree/develop/hermes-five/examples) directory to learn by doing.

## Key Features

* **🧠 High-level abstractions:** Control LEDs, sensors, buttons, servos and all kind of devices. Write expressive, async
  Rust code to control them.
* **🛜 Protocol-agnostic:** Serial, WiFi and Bluetooth supported (via Firmata).
* **🧩 Modular design:** Plug-and-play support for boards and devices. Arduino, ESP32, nodeMCU, Raspberry Pi, etc.
* **🕹️ Animation engine:** Interpolate servo movements, LED fades and more with ease.
* **🧪 Test-friendly:** Includes mocks to run and test logic without hardware.

_🖱️ Prefer a GUI over code? Try [Hermes-Studio](https://github.com/dclause/hermes-studio) - a visual programming
interface powered by Hermes-Five._

## Getting started

- Flash the
  compatible [Firmata Protocol client](https://github.com/firmata/arduino/blob/main/examples/StandardFirmataPlus/StandardFirmataPlus.ino)
  via Arduino IDE on your board.
- Create a new Rust project:

```shell
cargo new my_awesome_project
cd my_awesome_project
```

- Add this crate to your dependencies in the `Cargo.toml` file.

```toml
[dependencies]
hermes-five = "0.1.0"
```

- Modify your `src/main.rs` as needed (
  see [examples](https://github.com/dclause/hermes-five/tree/develop/hermes-five/examples) for inspiration).
- Start by exploring the [examples](https://github.com/dclause/hermes-five/tree/develop/hermes-five/examples) code,
  the [user documentation](https://dclause.github.io/hermes-five)
- or the [API documentation](https://docs.rs/hermes-five/latest)

> [!TIP]
> Feature flags:
>   - **libudev** -- (enabled by default) Activates `serialport` crate _libudev_ feature under-the-hood (required on
      Linux only for port listing).
>   - **serde** -- Enables serialize/deserialize capabilities for most entities.
>   - **mock** -- Provides mocked entities of all kinds (useful for tests mostly).

### Hello Hermes!

The following example shows the simplest possible program: from your computer, command a serially connected Arduino to
blink its built-in LED on pin 13.

```rust
use hermes_five::hardware::Board;
use hermes_five::devices::Led;

#[hermes_five::runtime]
async fn main() {

    // Register a new board 
    // (of type arduino + auto-detected serial port by default)
    let board = Board::start().unwrap();

    // When board communication is ready:
    board.on_ready(|board: Board| async move {

        // Register a LED on pin 13 (arduino embedded led).
        // Pin: 13; OFF by default
        let mut led = Led::new(&board, 13, false)?;

        // Blinks the LED every 500ms: indefinitely.
        led.blink(500);

        Ok(())
    });
}
```

## Examples

All available examples can be found in
the [examples](https://github.com/dclause/hermes-five/tree/develop/hermes-five/examples) folder.

To start an example, run the following command:

```
cargo run --example folder_examplename
```

To run an example in a file called `examples/folder/examplename.rs`, use the concatenation name
as `folder_examplename`.

If you want the "full" log output you can use:

```
RUST_LOG=DEBUG cargo run --example folder_examplename
```

## Roadmap

For details, see the full [roadmap](/roadmap.md): currently working
toward release 0.2

## Contribution

All contributions are more than welcome through [PR](https://github.com/dclause/hermes-five/pulls) and
the [issue queue](https://github.com/dclause/hermes-five/issues).

- Fork the repository
- Create a new branch: `git checkout -b feature-branch`
- Commit your changes: `git commit -am 'Add new feature'`
- Push to the branch: `git push origin feature-branch`
- Create a new Pull Request

**_The author does not claim to know everything about Rust programming or IoT, and all ideas are welcome as long as they
respect the project's original philosophy._**

## License

This project is licensed under the MIT License. See
the [LICENSE](/LICENSE) file for details.

## Contact

For support, please open an issue or reach out to the [author](https://github.com/dclause).
