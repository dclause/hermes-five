//! Defines pieces of hardware that can be remotely controlled through IO exchange messages.

mod board;
mod pca9685;
mod pin;

pub use board::Board;
pub use board::BoardEvent;
pub use pca9685::PCA9685;
pub use pin::*;

use crate::errors::Error;
use crate::errors::HardwareError::UnknownPin;
use crate::utils::Range;

use crate::protocols::IoProtocol;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// The low-level API exposed by all Hardware.
/// It is discouraged to use low-level API in favor of using devices directly.
pub trait LowLevelApi {
    /// Returns the name of the protocol implemented by the device or system.
    ///
    /// This is typically a static identifier used to differentiate between
    /// communication standards or protocol families (e.g., "Firmata", "Modbus").
    fn get_protocol_name(&self) -> &str;

    /// Returns the version string of the protocol being used.
    ///
    /// This helps ensure compatibility between different protocol implementations
    /// by explicitly stating the supported version (e.g., "2.5", "1.0.0").
    fn get_protocol_version(&self) -> &str;

    /// Returns the name of the firmware running on the target device.
    ///
    /// This is useful for diagnostics, tooling, or when working with
    /// firmware-specific features or quirks.
    fn get_firmware_name(&self) -> &str;

    /// Returns the version string of the firmware.
    ///
    /// This version is often used in conjunction with the firmware name to track
    /// compatibility, features, or bug fixes.
    fn get_firmware_version(&self) -> &str;

    // ########################################
    // Pins info

    /// Returns a map of all known pins, indexed by their numeric IDs.
    ///
    /// The returned [`HashMap`] contains every pin registered in the system,
    /// with the key being the pin's numeric identifier (`u8`) and the value
    /// being a reference-counted [`Pin`] instance.
    ///
    /// # Returns
    /// A `HashMap<u8, Arc<Pin>>` containing all known pins.
    ///
    /// [`Pin`]: crate::Pin
    fn get_pins(&self) -> &HashMap<u8, Arc<Pin>>;

    #[inline(always)]
    fn get_pin_value(&self, pin: u8) -> Result<u16, Error> {
        Ok(self.get_pin(pin)?.get_value())
    }

    // ########################################
    // Read/Write on pins

    /// Sets the [`PinMode`] of the specified pin by its numeric ID.
    ///
    /// This corresponds to the *[Set Pin Mode]* command in the [Firmata protocol].
    ///
    /// # Arguments
    /// * `pin` – The numeric ID (`u8`) of the target pin.
    /// * `mode` – The desired mode for the pin (e.g., `Input`, `Output`, `Pwm`, etc.).
    ///
    /// # Errors
    /// Returns [`Error`] if the operation fails or the pin/mode is unsupported.
    ///
    /// [Set Pin Mode]: https://github.com/firmata/protocol/blob/master/protocol.md#data-message-expansion
    /// [Firmata protocol]: https://github.com/firmata/protocol
    /// [`PinMode`]: crate::PinModeId
    /// [`Error`]: crate::Error
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
    /// Returns the I2C data for a given address.
    fn get_i2c_data(&self, address: u8) -> Arc<RwLock<Vec<I2CReply>>>;
}

pub trait LowLevelApiExt: LowLevelApi {
    /// Retrieves a reference-counted [`Pin`] by its numeric ID.
    ///
    /// # Arguments
    /// * `pin` – The numeric ID (`u8`) of the pin to retrieve.
    ///
    /// # Returns
    /// A shared [`Arc`] to the corresponding [`Pin`] if it exists.
    ///
    /// # Errors
    /// Returns [`Error::UnknownPin`] if no pin with the given ID is found.
    ///
    /// # Examples
    /// ```exclude
    /// let pin = board.get_pin(3)?; // Look up pin by ID
    /// ```
    ///
    /// [`Pin`]: crate::Pin
    /// [`Arc`]: std::sync::Arc
    /// [`Error::UnknownPin`]: crate::Error::UnknownPin
    fn get_pin<P: Into<PinIdOrName>>(&self, pin: P) -> Result<Arc<Pin>, Error> {
        let pin = pin.into();
        let pins = self.get_pins();
        match &pin {
            PinIdOrName::Id(id) => pins.get(id).cloned().ok_or(Error::from(UnknownPin { pin })),
            PinIdOrName::Name(name) => Ok(pins
                .iter()
                .find(|(_, pin)| pin.get_name() == name)
                .ok_or(Error::from(UnknownPin { pin }))?
                .1
                .clone()),
        }
    }
}

impl<T: LowLevelApi + ?Sized> LowLevelApiExt for T {}

/// You most likely don't need this function (outside this crate).
pub trait Hardware: LowLevelApi {
    /// Returns the protocol used.
    fn get_protocol(&self) -> Arc<dyn IoProtocol>;

    #[cfg(feature = "serde")]
    /// Sets the protocol.
    fn set_protocol(&mut self, protocol: Arc<dyn IoProtocol>);
}

pub trait Expander: Hardware {}
