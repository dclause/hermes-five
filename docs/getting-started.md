# Getting started

![PC-MEGA-serial.png](/communication/PC-MEGA-serial.png)

## Pre-requisites

The following procedure will assume you to have :

- [Rust cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed on your machine.
- An [Arduino](https://www.arduino.cc/en/hardware) board attached via the serial port of your computer.
- [StandardFirmataPlus.ino](https://github.com/firmata/arduino/blob/main/examples/StandardFirmataPlus/StandardFirmataPlus.ino)
  Arduino sketch installed on the board.   
  _This code is available by default in [Arduino IDE](https://www.arduino.cc/en/software) under the Firmata samples
  sketch menu._  
  _Uploading the sketch to the board needs to be done once only._

## Create a new project

- Create a new Rust project.

```shell
cargo new my_awesome_project
cd my_awesome_project
```

- Add this crate to your dependencies in the `Cargo.toml` file.

```toml
[dependencies]
hermes-five = { branch = "develop", git = "https://github.com/dclause/hermes-five" }
```

- Change `src/main.rs` file to the following.

```rust
use hermes_five::{Board, BoardEvent};
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

- Simply run your Rust program as usual.

```shell
cargo run
```

## Up and Running (or Troubleshooting)

You should see the embedded LED of your Arduino board blink.

**If any issue occurred, please refer to the [Troubleshooting](/troubleshooting) section.**

## What's Next?

* Start by exploring the [examples](https://github.com/dclause/hermes-five/tree/develop/hermes-five/examples) code.
* Learn more about the concepts behind Hermes-Five.
* Discover the API.
* [Share your code samples](https://github.com/dclause/hermes-five/pulls) with us by creating new examples.

