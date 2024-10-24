use std::collections::HashMap;
use std::fmt::Debug;

use crate::errors::HardwareError::UnknownPin;
use crate::errors::*;
use crate::protocols::{I2CReply, Pin, PinIdOrName};

/// Represents the hardware and internal data that a generic protocol handles.
///
/// This struct is hidden behind an `Arc<RwLock<Hardware>>` to allow safe concurrent access
/// and modification through the `Hardware` type. It encapsulates data relevant
/// to the protocol, such as pins and I2C communication data. Doing so allows us to set default
/// implementation for most communication methods in the [`Protocol`] trait.
///
/// # Fields
///
/// - `pins`: A vector of `Pin` instances, representing the hardware's pins.
/// - `i2c_data`: A vector of `I2CReply` instances, representing I2C communication data.
/// - `protocol_version`: A string indicating the version of the protocol.
/// - `firmware_name`: A string representing the name of the firmware.
/// - `firmware_version`: A string representing the version of the firmware.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hardware {
    /// A vector of `Pin` instances, representing the hardware's pins.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub pins: HashMap<u16, Pin>,
    /// A vector of `I2CReply` instances, representing I2C communication data.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub i2c_data: Vec<I2CReply>,
    /// A string indicating the version of the protocol.
    pub protocol_version: String,
    /// A string representing the name of the firmware.
    pub firmware_name: String,
    /// A string representing the version of the firmware.
    pub firmware_version: String,
}

impl Hardware {
    /// Retrieves a reference to a pin by its id or name.
    ///
    /// # Arguments
    /// * `pin`  - The index of the pin to retrieve.
    ///
    /// # Errors
    /// * `UnknownPin` - An `Error` returned if the pin index is out of bounds.
    pub fn get_pin<T: Into<PinIdOrName>>(&self, pin: T) -> Result<&Pin, Error> {
        let pin = pin.into();
        match &pin {
            PinIdOrName::Id(id) => self.pins.get(&id).ok_or(Error::from(UnknownPin { pin })),
            PinIdOrName::Name(name) => Ok(self
                .pins
                .iter()
                .find(|(_, &ref pin)| pin.name == *name)
                .ok_or(Error::from(UnknownPin { pin }))?
                .1),
        }
    }

    /// Retrieves a mutable reference to a pin by its id or name.
    ///
    /// # Arguments
    /// * `pin` - The index of the pin to retrieve.
    ///
    /// # Errors
    /// * `UnknownPin` - An `Error` returned if the pin index is out of bounds.
    pub fn get_pin_mut<T: Into<PinIdOrName>>(&mut self, pin: T) -> Result<&mut Pin, Error> {
        let pin = pin.into();
        match &pin {
            PinIdOrName::Id(id) => self
                .pins
                .get_mut(&id)
                .ok_or(Error::from(UnknownPin { pin })),
            PinIdOrName::Name(name) => Ok(self
                .pins
                .iter_mut()
                .find(|(_, &mut ref pin)| pin.name == *name)
                .ok_or(Error::from(UnknownPin { pin }))?
                .1),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mocks::hardware::create_test_hardware;

    #[test]
    fn test_get_pin_success() {
        assert_eq!(create_test_hardware().get_pin(3).unwrap().value, 3);
        assert_eq!(create_test_hardware().get_pin(11).unwrap().value, 11);
        assert_eq!(create_test_hardware().get_pin_mut(3).unwrap().value, 3);
        assert_eq!(create_test_hardware().get_pin_mut(11).unwrap().value, 11);
    }

    #[test]
    fn test_get_pin_error() {
        assert!(create_test_hardware().get_pin(66).is_err());
        assert!(create_test_hardware().get_pin_mut(66).is_err());
    }

    #[test]
    fn test_mutate_pin() {
        let mut hardware = create_test_hardware();
        assert_eq!(hardware.get_pin_mut(11).unwrap().value, 11);
        hardware.get_pin_mut(11).unwrap().value = 255;
        assert_eq!(hardware.get_pin_mut(11).unwrap().value, 255);
    }
}
