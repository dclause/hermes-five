use std::sync::Arc;

use parking_lot::RwLock;

use crate::protocols::{Error, I2CReply, Pin, UnknownPin};

pub type ProtocolHardware = Arc<RwLock<Hardware>>;

/// Represents the hardware and internal data a generic protocol is supposed to handle.
/// In an objet-oriented paradigm, that would be `Protocol` abstract class attributes we must ensure
/// every protocol has, because we rely on it.
/// In Rust, this is handle by this `ProtocolHardware` hardware structure we enforce a `Protocol`
/// implementation to have via the getter [`Protocol::get_hardware_mut()`].
/// This lets our `Protocol` trait to implement most of the protocol generically.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hardware {
    #[cfg_attr(feature = "serde", serde(skip))]
    pub pins: Vec<Pin>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub i2c_data: Vec<I2CReply>,
    pub protocol_version: String,
    pub firmware_name: String,
    pub firmware_version: String,
}

impl Hardware {
    pub fn get_pin_mut(&mut self, pin: u16) -> Result<&mut Pin, Error> {
        self.pins.get_mut(pin as usize).ok_or(UnknownPin { pin })
    }
}
