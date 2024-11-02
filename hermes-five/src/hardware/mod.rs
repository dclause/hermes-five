//! Defines pieces of hardware that can be remotely controlled through IO exchange messages.

mod board;
mod pca9685;

use crate::io::IoProtocol;
pub use board::Board;
pub use board::BoardEvent;
pub use pca9685::PCA9685;

pub trait Hardware {
    /// Returns  the protocol used.
    fn get_protocol(&self) -> Box<dyn IoProtocol>;
}
