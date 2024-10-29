//! Defines various protocols to control devices associated to boards.

mod data;
pub mod firmata;
mod protocol;
mod transports;

pub use data::*;
pub use firmata::*;
pub use protocol::*;
pub use transports::*;
