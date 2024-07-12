use std::sync::Arc;

use parking_lot::RwLock;

use crate::protocols::{Error, I2CReply, Pin, UnknownPin};

pub type ProtocolHardware = Arc<RwLock<Hardware>>;

/// Represents the hardware and internal data a generic protocol is supposed to handle.
///
/// This is made to be hidden being an Arc<RwLock>> via the [`ProtocolHardware`] type, so the [`Protocol`]
/// can implement of the protocol functions generically.
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
