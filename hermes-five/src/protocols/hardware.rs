use crate::errors::*;
use crate::errors::HardwareError::UnknownPin;
use crate::protocols::{I2CReply, Pin};

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
    pub pins: Vec<Pin>,
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
    /// Retrieves a reference to a pin by its index.
    ///
    /// # Arguments
    /// * `pin`  - The index of the pin to retrieve.
    ///
    /// # Errors
    /// * `UnknownPin` - An `Error` returned if the pin index is out of bounds.
    pub fn get_pin(&self, pin: u16) -> Result<&Pin, Error> {
        self.pins
            .get(pin as usize)
            .ok_or(Error::from(UnknownPin { pin }))
    }

    /// Retrieves a mutable reference to a pin by its index.
    ///
    /// # Arguments
    /// * `pin` - The index of the pin to retrieve.
    ///
    /// # Errors
    /// * `UnknownPin` - An `Error` returned if the pin index is out of bounds.
    pub fn get_pin_mut(&mut self, pin: u16) -> Result<&mut Pin, Error> {
        self.pins
            .get_mut(pin as usize)
            .ok_or(Error::from(UnknownPin { pin }))
    }
}

#[cfg(test)]
mod tests {
    use crate::protocols::{Hardware, Pin};

    fn test_hardware() -> Hardware {
        Hardware {
            pins: vec![
                Pin {
                    id: 0,
                    value: 0,
                    ..Pin::default()
                },
                Pin {
                    id: 1,
                    value: 1,
                    ..Pin::default()
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn test_get_pin_success() {
        assert_eq!(test_hardware().get_pin(0).unwrap().value, 0);
        assert_eq!(test_hardware().get_pin(1).unwrap().value, 1);
        assert_eq!(test_hardware().get_pin_mut(0).unwrap().value, 0);
        assert_eq!(test_hardware().get_pin_mut(1).unwrap().value, 1);
    }

    #[test]
    fn test_get_pin_error() {
        assert!(test_hardware().get_pin(2).is_err());
        assert!(test_hardware().get_pin_mut(2).is_err());
    }

    #[test]
    fn test_mutate_pin() {
        let mut hardware = test_hardware();
        assert_eq!(hardware.get_pin_mut(1).unwrap().value, 1);
        hardware.get_pin_mut(1).unwrap().value = 3;
        assert_eq!(hardware.get_pin_mut(1).unwrap().value, 3);
    }
}
