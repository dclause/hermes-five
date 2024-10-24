#![doc(html_root_url = "https://docs.rs/hermes-five/0.1.0-beta")]

#[cfg(test)]
extern crate self as hermes_five;

pub use board::Board;
pub use board::BoardEvent;
pub use hermes_macros::runtime;

pub mod animation;
mod board;
pub mod devices;
pub mod errors;
#[cfg(any(test, feature = "mocks"))]
pub mod mocks;
pub mod protocols;
pub mod utils;
