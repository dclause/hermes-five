#[cfg(test)]
extern crate self as hermes_five;

pub use board::Board;
pub use board::BoardEvent;
// Expose hermes_macros::runtime to be used as `#[hermes_five::runtime]`
pub use hermes_macros::runtime;

pub mod animation;
mod board;
pub mod devices;
pub mod errors;
pub mod protocols;
// mod storage;
#[cfg(feature = "mocks")]
pub mod mocks;
pub mod utils;
