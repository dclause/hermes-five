#![doc(html_root_url = "https://docs.rs/hermes-five/0.1.0")]

//! <h1 align="center">HERMES-FIVE - The Rust Robotics & IoT Platform</h1>
//! <div style="text-align:center;font-style:italic;">Hermes-Five is an open-source IoT and Robotics programming framework - written in Rust.</div>
//! <br/>
//! <img height="0" style="float:right;height:200px!important;" alt="Schema sample of blinking led using Arduino UNO" src="https://github.com/dclause/hermes-five/blob/develop/docs/public/examples/led/led-blink.gif?raw=true" />
//!
//! # Documentation
//!
//! This is the API documentation.<br/>
//! To read more detailed explanations, visit the [user documentation](https://dclause.github.io/hermes-five).<br/>
//! To see the code in action, visit the [examples](https://github.com/dclause/hermes-five/tree/develop/hermes-five/examples) directory.
//!
//! # Features
//!
//! **Hermes-Five** is a Rust library designed to "remotely" control Arduino (or compatible) boards as well as all types
//! of input/output devices (led, servo, button, sensors, etc.) connected to it.<br/>
//! It can be compared to _[Johnny-Five](https://johnny-five.io/)_ in the javascript ecosystem.
//!
//! - Define remotely controllable [`Board`](hardware::Board) (Arduino currently)
//! - Control boards though an [`IoProtocol`](io::IoProtocol) connection ([`Serial`](io::Serial) for the moment)
//! - Remote control all types of [`Device`](devices::Device)s such as [`Output`](devices::Output)s (LED, servo, etc.) or [`Input`](devices::Input)s (button, switch, sensors,
//! - etc.) individually
//! - Create and play [`Animation`](animations::Animation) with auto-interpolate movements
//!
//! **_If you wish to do the same with absolutely no code via a nice-and-shiny interface, please consult the [Hermes-Studio](https://github.com/dclause/hermes-studio) project._**
//!
//! # Prerequisites
//!
//! - To run the [examples](https://github.com/dclause/hermes-five/tree/develop/hermes-five/examples) provided, you will at least an Arduino board attached via the serial port of your computer (or the machine running your code).<br/>
//! - [StandardFirmataPlus.ino](https://github.com/firmata/arduino/blob/main/examples/StandardFirmataPlus/StandardFirmataPlus.ino) Arduino sketch **MUST** be installed on the board.
//!   _This code is available by default in Arduino IDE under the Firmata samples sketch menu._
//!   _Uploading the sketch to the board needs to be done once only._
//!
//! # Getting Started
//!
//! - Install the compatible [Firmata Protocol client](https://github.com/firmata/arduino/blob/main/examples/StandardFirmataPlus/StandardFirmataPlus.ino) on your Arduino board.
//!
//! - Add the following to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! hermes-five = "0.1.0"
//! ```
//!
//! - Start writing your HERMES code: see the [examples](https://github.com/dclause/hermes-five/tree/develop/hermes-five/examples) directory for more examples.
//!
//! The following code demonstrates the simplest program we could imagine: blink the Arduino embedded led on pin 13.
//! ```rust
//! use hermes_five::hardware::{Board, BoardEvent};
//! use hermes_five::devices::Led;
//!
//! #[hermes_five::runtime]
//! async fn main() {
//!
//!     // Register a new board.
//!     // (of type arduino + auto-detected serial port by default)
//!     let board = Board::run();
//!
//!     // When board communication is ready:
//!     board.on(BoardEvent::OnReady, |board: Board| async move {
//!
//!         // Register a LED on pin 13 (arduino embedded led).
//!         // Pin: 13; OFF by default
//!         let mut led = Led::new(&board, 13, false)?;
//!
//!         // Blinks the LED every 500ms: indefinitely.
//!         led.blink(500);
//!
//!         Ok(())
//!     });
//! }
//! ```
//!
//! # Feature flags
//!
//! - **libudev** -- (enabled by default) Activates `serialport` crate _libudev_ feature under-the-hood (required on Linux only for port listing).
//! - **serde** -- Enables serialize/deserialize capabilities for most entities.
//! - **mock** -- Provides mocked entities of all kinds (useful for tests mostly).

#[cfg(test)]
extern crate self as hermes_five;

pub mod animations;
pub mod devices;
pub mod errors;
pub mod hardware;
pub mod io;
#[cfg(any(test, feature = "mocks"))]
pub mod mocks;
pub mod utils;

pub use hermes_macros::runtime;
