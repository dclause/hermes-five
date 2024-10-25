//! Defines protocols to discuss and control hardware associated to boards.

mod data;
pub mod firmata;
mod plugin;
mod transports;

// Re-exports.
pub use data::*;
pub use firmata::*;
pub use plugin::*;
pub use transports::*;
// --
