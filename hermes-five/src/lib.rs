#![doc(html_root_url = "https://docs.rs/hermes-five/0.1.0-beta")]

//! # HERMES-FIVE - The Rust Robotics & IoT Platform
//! ![PC-MEGA-serial.png](/docs/public/communication/PC-MEGA-serial.png)
//!
//! ## Features
//!
//! The goal of this library is to expose an API to talk with electronic devices in Rust. It allows you to seamlessly remote control Arduino (or compatible) boards as well as all
//! types of input/output devices (led, servo, button, sensors, etc.).   
//! It can be compared to _Johnny-Five_ in the javascript ecosystem.
//!
//! * Configure your remote controllable [`Board`] (Arduino currently)
//! * Control boards though a [`IoData`] connection ([`SerialFirmata`] for the moment)
//! * Remote control all types of [`Device`](s) such as [`Output`](s) (LED, servo, etc.) or [`Input`](s) (button, switch, sensors,
//! * etc.) individually
//! * Create and play [`animation::Animation`] to interpolate movements
//!
//! ## Prerequisites
//!
//! * To run the examples provided, you will at least an Arduino board attached via the serial port of your computer (or the machine running your code).
//! * [StandardFirmataPlus.ino](https://github.com/firmata/arduino/blob/main/examples/StandardFirmataPlus/StandardFirmataPlus.ino) Arduino sketch **MUST** be installed on the board.
//!   _This code is available by default in Arduino IDE under the Firmata samples sketch menu._
//!   _Uploading the sketch to the board needs to be done once only._
//!
//! ## Getting Started
//!
//! * Add the following to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! hermes-five = "0.1.0"
//! ```
//! * Start writing your HERMES code: see the [examples](https://github.com/dclause/hermes-five/tree/develop/hermes-five/examples) directory for more examples.
//!
//! ## Feature flags
//! * **libudev** -- (enabled by default) Activates `serialport` crate _libudev_ feature under-the-hood (required on Linux only for port listing).
//! * **serde** -- Enables serialize/deserialize capabilities for most entities.
//! * **mock** -- Provides mocked entities of all kinds (useful for tests mostly).
#[cfg(test)]
extern crate self as hermes_five;

pub use hermes_macros::runtime;

pub mod animations;
pub mod devices;
pub mod errors;
pub mod hardware;
pub mod io;
#[cfg(any(test, feature = "mocks"))]
pub mod mocks;
// pub mod protocols;
pub mod utils;
