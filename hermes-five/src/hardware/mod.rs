//! Defines pieces of hardware that can be remotely controlled through IO exchange messages.

mod board;
mod pca9685;

use crate::io::{IoProtocol, IO};
pub use board::Board;
pub use board::BoardEvent;
pub use pca9685::PCA9685;

/// You most likely don't need this function (outside this crate).
pub trait Hardware: IO {
    /// Returns the protocol name.
    fn get_protocol_name(&self) -> &str {
        self.get_protocol().get_name()
    }

    /// Returns the protocol used.
    fn get_protocol(&self) -> Box<dyn IoProtocol>;

    /// Sets the protocol.
    /// @todo remove this when hermes_studio finds a way around.
    fn set_protocol(&mut self, protocol: Box<dyn IoProtocol>);
}

pub trait Controller: Hardware + IoProtocol {}
