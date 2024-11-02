# Hermes-Five

[![License](https://img.shields.io/github/license/dclause/hermes-five?color=success)](/LICENSE)
[![Documentation](https://img.shields.io/badge/documentation-_online-success)](https://dclause.github.io/hermes-five/)
[![Build Status](https://github.com/dclause/hermes-five/workflows/Build/badge.svg)](https://github.com/dclause/hermes-five/actions/workflows/build.yml)
[![Test Status](https://github.com/dclause/hermes-five/workflows/Test/badge.svg)](https://github.com/dclause/hermes-five/actions/workflows/test.yml)
[![Code Coverage](https://codecov.io/gh/dclause/hermes-five/graph/badge.svg?token=KF8EFDUQ7A)](https://codecov.io/gh/dclause/hermes-five)

### The Rust Robotics & IoT Platform

<img align="right" style="height:200px" alt="Schema sample of blinking led using Arduino UNO" src="/docs/public/examples/led/led-blink.gif?raw=true" />

**_Hermes-Five_ is an open-source, [Firmata Protocol](https://github.com/firmata/protocol)-based, IoT and Robotics
programming framework - written in Rust.**

_The ultimate goal of this project is to mimic the functionalities of [Johnny-Five](https://johnny-five.io/) framework
where it finds its inspiration (hence the name) - but using Rust. That being said, the project is done
in [my spare time](https://github.com/dclause) and
does not intend
to compete with any other solutions you might want to try,_

## Documentation

Documentation is available to you in three forms:

- The [user documentation](https://dclause.github.io/hermes-five) for general knowledge.
- The [API documentation](https://docs.rs/hermes-five/latest) for developer references.
- The [examples](https://github.com/dclause/hermes-five/tree/develop/hermes-five/examples) directory shows you code in
  action.

## Getting started

In a nutshell:

- Install the
  compatible [Firmata Protocol client](https://github.com/firmata/arduino/blob/main/examples/StandardFirmataPlus/StandardFirmataPlus.ino)
  on your Arduino board.
- Create a new Rust project:

```shell
cargo new my_awesome_project
cd my_awesome_project
```

- Add this crate to your dependencies in the `Cargo.toml` file.

```toml
[dependencies]
hermes-five = "0.1.0-beta"
```

- Change `src/main.rs` file as need (
  see [examples](https://github.com/dclause/hermes-five/tree/develop/hermes-five/examples))

> [!TIP]
> Available feature flags are:
>   - **libudev** -- (enabled by default) Activates `serialport` crate _libudev_ feature under-the-hood (required on
      Linux only for port listing).
>   - **serde** -- Enables serialize/deserialize capabilities for most entities.
>   - **mock** -- Provides mocked entities of all kinds (useful for tests mostly).

- Start by exploring the [examples](https://github.com/dclause/hermes-five/tree/develop/hermes-five/examples) code,
  the [user documentation](https://dclause.github.io/hermes-five)
- or the [API documentation](https://docs.rs/hermes-five/latest)

## Features

**Hermes-Five** is a Rust library designed to "remotely" control Arduino (or compatible) boards as well as all types of
input/output devices (led, servo, button, sensors, etc.) connected to it. <br/>
It can be compared to _[Johnny-Five](https://johnny-five.io/)_ in the javascript ecosystem.
**Hermes-Five** is a Rust library designed to "remotely" control Arduino (or compatible boards) using Rust code.

* Define remotely controllable `Board` (Arduino currently)
* Control boards though an `IoProtocol` connection (`Serial` for the moment)
* Control all types of `Device` such as `Output` (LED, servo, etc.) or `Input` (button, switch, sensors,
* etc.) individually
* Create and play `Animation` with auto-interpolate movements

**_If you wish to do the same with absolutely no code via a nice-and-shiny interface, please consult
the [Hermes-Studio](https://github.com/dclause/hermes-studio) project._**

### Hello Hermes!

The following code demonstrates the simplest program we could imagine: blink the Arduino embedded led on pin 13.

```rust
use hermes_five::hardware::{Board, BoardEvent};
use hermes_five::devices::Led;

#[hermes_five::runtime]
async fn main() {

    // Register a new board.
    // (of type arduino + auto-detected serial port by default)
    let board = Board::run();

    // When board communication is ready:
    board.on(BoardEvent::OnReady, |board: Board| async move {

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
toward release 0.1

In short:

- [version 0.1](/roadmap.md#release-version-01): proof-of-concept;
    - basics concepts and underlying requirements (events, tasks, multi-tasking, etc.)
    - API syntax
    - Firmata communication to read/write data for basic input/ouput
- [version 0.2](/roadmap.md#release-version-02): PoC of swapping the underlying bricks
    - multi-protocol compatibility: serial, wifi, etc.
    - multi-board compatibility: arduino, raspberry, etc.
    - clarify the hardware layer (ex. Board/PCA9685/IOExpanders for devices)

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