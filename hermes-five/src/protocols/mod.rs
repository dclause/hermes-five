pub mod constants;
mod remote;

pub use remote::RemoteIo;

use crate::errors::Error;
use crate::hardware::LowLevelApi;
use crate::utils::private::TraitToAny;
use dyn_clone::DynClone;
use std::fmt::{Debug, Display};

// Makes a Box<dyn IoPlugin> clone (used for Board cloning).
dyn_clone::clone_trait_object!(IoProtocol);

/// Defines the trait all protocols must implement.
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait IoProtocol: LowLevelApi + DynClone + Send + Sync + Debug + Display + TraitToAny {
    /// Opens the communication using the underlying protocol.
    fn open(&self) -> Result<(), Error>;

    /// Gracefully shuts down the communication.
    fn close(&self) -> Result<(), Error>;

    /// Sets the analog reporting `state` of the specified analog `pin`.
    ///
    /// When activated, the pin will send its value periodically. The value will be stored in the IoProtocol synced data.
    /// <https://github.com/firmata/protocol/blob/master/protocol.md>
    fn report_analog(&self, channel: u8, state: bool) -> Result<(), Error>;

    /// Sets the digital reporting `state` of the specified digital `pin`.
    ///
    /// This will activate the reporting of all pins in port (hence the pin will send us its value periodically)
    /// <https://github.com/firmata/protocol/blob/master/protocol.md>
    fn report_digital(&self, pin: u8, state: bool) -> Result<(), Error>;

    /// Set the sampling `interval` (in ms).
    ///
    /// The sampling interval sets how often analog data and i2c data is reported to the
    /// client. The default for the arduino implementation is 19ms. This means that every
    /// 19ms analog data will be reported and any i2c devices with read continuous mode
    /// will be read.
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#sampling-interval>
    fn sampling_interval(&self, interval: u16) -> Result<(), Error>;
}
