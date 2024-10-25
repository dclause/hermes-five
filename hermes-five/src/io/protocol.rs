//! Defines protocols to discuss and control hardware associated to boards.

use std::any::type_name;
use std::fmt::{Debug, Display};
use std::sync::Arc;

use crate::errors::*;
use crate::io::firmata::Firmata;
use crate::io::{IoData, PinModeId};
use crate::utils::Range;
use dyn_clone::DynClone;
use parking_lot::RwLock;

// Makes a Box<dyn IoPlugin> clone (used for Board cloning).
dyn_clone::clone_trait_object!(IoProtocol);

/// Defines the trait all protocols must implement.
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait IoProtocol: DynClone + Send + Sync + Debug + Display {
    // ########################################
    // Inner data related functions

    fn get_data(&self) -> &Arc<RwLock<IoData>>;

    /// Returns the protocol name (used for Display only)
    fn get_protocol_name(&self) -> &'static str {
        type_name::<Self>().split("::").last().unwrap()
    }

    // ########################################
    // Functions specifically bound to the protocol.

    /// Open the communication using the underlying protocol.
    fn open(&mut self) -> Result<(), Error>;
    /// Gracefully shuts down the communication.
    fn close(&mut self) -> Result<(), Error>;
    /// Checks if the communication is opened using the underlying protocol.
    fn is_connected(&self) -> bool;

    // ########################################
    // Read/Write on pins

    /// Set the `mode` of the specified `pin`.
    ///
    /// https://github.com/firmata/protocol/blob/master/protocol.md#data-message-expansion
    fn set_pin_mode(&mut self, pin: u16, mode: PinModeId) -> Result<(), Error>;

    /// Write `level` to the digital `pin`.
    ///
    /// Send an DIGITAL_MESSAGE (0x90 - set digital value).
    /// https://github.com/firmata/protocol/blob/master/protocol.md#message-types
    fn digital_write(&mut self, pin: u16, level: bool) -> Result<(), Error>;

    /// Write `level` to the analog `pin`.
    ///
    /// Send an ANALOG_MESSAGE (0xE0 - set analog value).
    /// https://github.com/firmata/protocol/blob/master/protocol.md#message-types
    fn analog_write(&mut self, pin: u16, level: u16) -> Result<(), Error>;

    /// Activates reporting `state` of the specified analog `pin`.
    ///
    /// When activated, the pin will send its value periodically. The value will be stored in the IoProtocol synced data.
    /// ```no_run
    /// use hermes_five::hardware::Board;
    /// let mut board = Board::default();
    /// board.report_analog(0, true).expect("");
    /// board.get_io().get_pin("A0").expect("").value;
    /// ```
    fn report_analog(&mut self, channel: u8, state: bool) -> Result<(), Error>;

    /// Set the digital reporting `state` of the specified digital `pin`.
    ///
    /// This will activate the reporting of all pins in port (hence the pin will send us its value periodically)
    /// https://github.com/firmata/protocol/blob/master/protocol.md
    fn report_digital(&mut self, pin: u16, state: bool) -> Result<(), Error>;

    /// Set the sampling interval (in ms).
    ///
    /// The sampling interval sets how often analog data and i2c data is reported to the
    /// client. The default for the arduino implementation is 19ms. This means that every
    /// 19ms analog data will be reported and any i2c devices with read continuous mode
    /// will be read.
    /// https://github.com/firmata/protocol/blob/master/protocol.md#sampling-interval
    fn sampling_interval(&mut self, interval: u16) -> Result<(), Error>;

    // ########################################
    // I2C

    /// Configure the `delay` in microseconds for I2C devices that require a delay between when the
    /// register is written to and the data in that register can be read.
    fn i2c_config(&mut self, delay: u16) -> Result<(), Error>;
    /// Read `size` bytes from I2C device at the specified `address`.
    fn i2c_read(&mut self, address: i32, size: i32) -> Result<(), Error>;
    /// Write `data` to the I2C device at the specified `address`.
    fn i2c_write(&mut self, address: i32, data: &[u8]) -> Result<(), Error>;

    // ########################################
    // SERVO

    /// Sends a SERVO_CONFIG command (0x70 - configure servo)
    /// https://github.com/firmata/protocol/blob/master/servos.md
    fn servo_config(&mut self, pin: u16, pwm_range: Range<u16>) -> Result<(), Error>;
}

#[cfg(not(tarpaulin_include))]
impl Default for Box<dyn IoProtocol> {
    fn default() -> Self {
        Box::new(Firmata::default())
    }
}
