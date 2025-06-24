//! Defines various protocols to control devices associated to boards.

use crate::errors::Error;
use crate::utils::Range;
use parking_lot::RwLock;
use std::sync::Arc;

mod constants;
mod data;
mod protocols;
mod transports;

pub use data::*;
pub use protocols::*;
pub use transports::*;

pub trait IO {
    // ########################################
    // Inner data related functions

    /// Returns a protected arc to the inner [`IoData`].
    fn get_io(&self) -> &Arc<RwLock<IoData>>;

    /// Checks if the communication is opened using the underlying protocol.
    fn is_connected(&self) -> bool;

    // ########################################
    // Read/Write on pins

    /// Sets the `mode` of the specified `pin`.
    ///
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#data-message-expansion>
    fn set_pin_mode(&self, pin: u8, mode: PinModeId) -> Result<(), Error>;

    /// Writes `level` to the digital `pin`.
    ///
    /// Send an DIGITAL_MESSAGE (0x90 - set digital value).
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#message-types>
    fn digital_write(&self, pin: u8, level: bool) -> Result<(), Error>;

    /// Writes `level` to the analog `pin`.
    ///
    /// Send an ANALOG_MESSAGE (0xE0 - set analog value).
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#message-types>
    fn analog_write(&self, pin: u8, level: u16) -> Result<(), Error>;

    /// Reads the digital `pin` value.
    fn digital_read(&self, pin: u8) -> Result<bool, Error>;

    /// Reads the analog `pin` value.
    fn analog_read(&self, pin: u8) -> Result<u16, Error>;

    // ########################################
    // SERVO

    /// Sends a SERVO_CONFIG command (0x70 - configure servo)
    /// <https://github.com/firmata/protocol/blob/master/servos.md>
    fn servo_config(&self, pin: u8, pwm_range: Range<u16>) -> Result<(), Error>;

    // ########################################
    // I2C

    /// Configures the `delay` in microseconds for I2C devices that require a delay between when the
    /// register is written to and the data in that register can be read.
    fn i2c_config(&self, delay: u16) -> Result<(), Error>;
    /// Reads `size` bytes from I2C device at the specified `address`.
    fn i2c_read(&self, address: u8, size: u16) -> Result<(), Error>;
    /// Writes `data` to the I2C device at the specified `address`.
    fn i2c_write(&self, address: u8, data: &[u16]) -> Result<(), Error>;
}
