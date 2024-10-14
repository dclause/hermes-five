# Hermes-Five

### The Rust Robotics & IoT Platform

[![License](https://img.shields.io/github/license/dclause/hermes-five)](https://github.com/dclause/hermes-five/blob/develop/LICENSE)
[![Documentation](https://img.shields.io/badge/documentation-_online-green)](https://dclause.github.io/hermes-five/)
[![Build Status](https://github.com/dclause/hermes-five/workflows/Build/badge.svg)](https://github.com/dclause/hermes-five/actions/workflows/build.yml)
[![Test Status](https://github.com/dclause/hermes-five/workflows/Test/badge.svg)](https://github.com/dclause/hermes-five/actions/workflows/test.yml)
[![Code Coverage](https://codecov.io/gh/dclause/hermes-five/graph/badge.svg?token=KF8EFDUQ7A)](https://codecov.io/gh/dclause/hermes-five)

<img align="right" style="height:200px" alt="Schema sample of blinking led using Arduino UNO" src="https://github.com/dclause/hermes-five/blob/develop/docs/public/examples/led/led-blink.gif?raw=true" />

**_Hermes-Five_ is an Open Source, [Firmata Protocol](https://github.com/firmata/protocol)-based, IoT and Robotics
programming framework - written in Rust.**

_The ultimate goal of this project is to mimic the functionalities of [Johnny-Five](https://johnny-five.io/) framework
where it finds its inspiration (hence the name) - but using Rust. That being said, the project is done
in [my spare time](https://github.com/dclause) and
does not intend
to compete with any other solutions you might want to try,_

## Documentation

To check out docs, visit [Github Pages](https://dclause.github.io/hermes-five).

## Features

**Hermes-Five** is a Rust library designed to remotely control Arduino (other supported boards to come) using Rust code.

**_If you wish to control your Hermes compatible robot using a UI (no-code), please consult
the [HermesStudio](https://github.com/dclause/hermes-studio)
project._**

### Hello Hermes!

The following code demonstrates the simplest program we could imagine: blink the Arduino embedded led on pin 13.

```rust
use hermes_five::{Board, BoardEvent};
use hermes_five::devices::{Actuator, Led};

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

## Instructions

- Install the
  compatible [Firmata Protocol client](https://github.com/firmata/arduino/blob/main/examples/StandardFirmataPlus/StandardFirmataPlus.ino)
  on your Arduino board.
- Create a new Rust project:

```
cargo new my_project
cd my_project
```

- Add this crate to your dependencies in the _Cargo.toml_
  file

```
[dependencies]
hermes-five = { branch = "develop", git = "https://github.com/dclause/hermes-five" }
```

- Start by exploring the [examples](https://github.com/dclause/hermes-five/tree/develop/hermes-five/examples) code and
  the [project documentation](https://dclause.github.io/hermes-five).

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

For details, see the full [roadmap](https://github.com/dclause/hermes-five/blob/develop/roadmap.md): currently working
toward release 0.1

## Contribution

All contributions are more than welcome through [PR](https://github.com/dclause/hermes-five/pulls) and
the [issue queue](https://github.com/dclause/hermes-five/issues).

- Fork the repository
- Create a new branch (git checkout -b feature-branch)
- Commit your changes (git commit -am 'Add new feature')
- Push to the branch (git push origin feature-branch)
- Create a new Pull Request

**_The author does not claim to know everything about Rust programming or IoT, and all ideas are welcome as long as they
respect the project's original philosophy._**

## License

This project is licensed under the MIT License. See
the [LICENSE](https://github.com/dclause/hermes-five/blob/develop/LICENSE) file for details.

## Contact

For support, please open an issue or reach out to the [author](https://github.com/dclause).