//! Defines protocols to discuss and control hardware associated to boards.

use std::any::type_name;
use std::fmt::{Debug, Display};

use crate::errors::Error;
use crate::io::firmata::FirmataIo;
use crate::io::IO;
use dyn_clone::DynClone;

// Makes a Box<dyn IoPlugin> clone (used for Board cloning).
dyn_clone::clone_trait_object!(IoProtocol);

/// Defines the trait all protocols must implement.
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait IoProtocol: IO + DynClone + Send + Sync + Debug + Display {
    /// Returns the protocol name.
    fn get_name(&self) -> &'static str {
        type_name::<Self>().split("::").last().unwrap()
    }

    /// Opens the communication using the underlying protocol.
    fn open(&mut self) -> Result<(), Error>;

    /// Gracefully shuts down the communication.
    fn close(&mut self) -> Result<(), Error>;

    ///  Sets the analog reporting `state` of the specified analog `pin`.
    ///
    /// When activated, the pin will send its value periodically. The value will be stored in the IoProtocol synced data.
    /// ```no_run
    /// use hermes_five::hardware::{Board, Hardware};
    /// use hermes_five::io::IO;
    /// let mut board = Board::default();
    /// board.get_protocol().report_analog(0, true).expect("");
    /// board.get_io().get_pin("A0").expect("").value;
    /// ```
    fn report_analog(&mut self, channel: u8, state: bool) -> Result<(), Error>;

    /// Sets the digital reporting `state` of the specified digital `pin`.
    ///
    /// This will activate the reporting of all pins in port (hence the pin will send us its value periodically)
    /// <https://github.com/firmata/protocol/blob/master/protocol.md>
    fn report_digital(&mut self, pin: u8, state: bool) -> Result<(), Error>;

    /// Set the sampling `interval` (in ms).
    ///
    /// The sampling interval sets how often analog data and i2c data is reported to the
    /// client. The default for the arduino implementation is 19ms. This means that every
    /// 19ms analog data will be reported and any i2c devices with read continuous mode
    /// will be read.
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#sampling-interval>
    fn sampling_interval(&mut self, interval: u16) -> Result<(), Error>;
}

#[cfg(not(tarpaulin_include))]
impl Default for Box<dyn IoProtocol> {
    fn default() -> Self {
        Box::new(FirmataIo::default())
    }
}
