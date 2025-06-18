// ***********
// All information are relative to PCF8575 datasheets:
// https://www.ti.com/lit/ds/symlink/pcf8575.pdf

use crate::errors::{Error, HardwareError};
use crate::hardware::{Board, Expander, Hardware};
use crate::io::{IoData, IoProtocol, Pin, PinMode, PinModeId, IO};
use crate::utils::Range;
use parking_lot::RwLock;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct PCF8575 {
    // Address (default 0x40).
    address: u8,
    // Sampling interval (default: 16ms)
    interval: u16,

    // ########################################
    // # Volatile utility data.
    #[cfg_attr(feature = "serde", serde(skip))]
    data: Arc<RwLock<IoData>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    protocol: Box<dyn IoProtocol>,
}

impl PCF8575 {
    fn _build_pca9685_data() -> IoData {
        let mut data = IoData {
            pins: Default::default(),
            i2c_data: vec![],
            digital_reported_pins: vec![],
            analog_reported_channels: vec![],
            protocol_version: "PCF8575".to_string(),
            firmware_name: "PCF8575".to_string(),
            firmware_version: "n/a".to_string(),
            connected: false,
        };

        for id in 0..16 {
            data.pins.insert(
                id,
                Pin {
                    id,
                    name: format!("D{}", id),
                    mode: Default::default(),
                    supported_modes: vec![
                        PinMode {
                            id: PinModeId::OUTPUT,
                            resolution: 1,
                        },
                        PinMode {
                            id: PinModeId::INPUT,
                            resolution: 1,
                        },
                        PinMode {
                            id: PinModeId::UNSUPPORTED,
                            resolution: 0,
                        },
                    ],
                    channel: None,
                    value: 0,
                },
            );
        }

        data
    }

    pub fn default(board: &Board) -> Result<Self, Error> {
        PCF8575::new(board, 0x20)
    }

    pub fn new(board: &dyn Hardware, address: u8) -> Result<Self, Error> {
        let protocol = board.get_protocol();
        let mut expander = Self {
            address,
            interval: 16,
            data: Arc::new(RwLock::new(PCF8575::_build_pca9685_data())),
            protocol,
        };
        IoProtocol::open(&mut expander)?;
        Ok(expander)
    }

    pub fn get_address(&self) -> u8 {
        self.address
    }

    ///  Set all pins to HIGH.
    fn reset(&mut self) -> Result<(), Error> {
        self.i2c_write(self.address, &[0x00FF, 0x00FF])
    }
}

impl Expander for PCF8575 {}

impl Hardware for PCF8575 {
    fn get_protocol(&self) -> Box<dyn IoProtocol> {
        Box::new(self.clone())
    }

    /// @todo remove this when hermes_studio finds a way around.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn set_protocol(&mut self, protocol: Box<dyn IoProtocol>) {
        self.protocol = protocol;
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl IoProtocol for PCF8575 {
    fn open(&mut self) -> Result<(), Error> {
        self.i2c_config(0)?;
        self.reset()?;
        self.data.write().connected = true;
        Ok(())
    }

    fn close(&mut self) -> Result<(), Error> {
        self.reset()?;
        self.data.write().connected = false;
        Ok(())
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn report_analog(&mut self, _: u8, _: bool) -> Result<(), Error> {
        Err(Error::NotImplemented)
    }

    fn report_digital(&mut self, _: u8, _: bool) -> Result<(), Error> {
        Err(Error::NotImplemented)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn sampling_interval(&mut self, interval: u16) -> Result<(), Error> {
        self.interval = interval;
        Ok(())
    }
}

impl IO for PCF8575 {
    fn get_io(&self) -> &Arc<RwLock<IoData>> {
        &self.data
    }

    fn is_connected(&self) -> bool {
        self.data.read().connected
    }

    fn set_pin_mode(&mut self, pin: u8, mode: PinModeId) -> Result<(), Error> {
        let mut lock = self.data.write();
        let pin_instance = lock.get_pin_mut(pin)?;
        let _mode = pin_instance
            .supports_mode(mode)
            .ok_or(HardwareError::IncompatiblePin {
                pin,
                mode,
                context: "try to set pin mode",
            })?;
        pin_instance.mode = _mode;

        // Nothing to do on the actual device.

        Ok(())
    }

    fn digital_write(&mut self, pin: u8, level: bool) -> Result<(), Error> {
        let mut value: [u16; 2] = [0, 0];

        {
            let mut lock = self.data.write();

            // Check if pin exists
            let pin_instance = lock.get_pin_mut(pin)?;

            // Store the value we will write to the current pin.
            pin_instance.value = if level { 0xFF } else { 0x00 };

            for port in 0..2 {
                // Loop through all 8 pins of the current "port" to concatenate their value.
                // For instance 01100000 will set to 1 the pin 1 and 2 or current port.
                for i in 0..8 {
                    if lock.get_pin(8 * port + i)?.value != 0 {
                        value[port as usize] |= 1 << i
                    }
                }
            }
        }

        self.protocol.i2c_write(self.address, &value)
    }

    fn analog_write(&mut self, _: u8, _: u16) -> Result<(), Error> {
        Err(Error::NotImplemented)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn digital_read(&mut self, _: u8) -> Result<bool, Error> {
        Err(Error::NotImplemented)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn analog_read(&mut self, _: u8) -> Result<u16, Error> {
        Err(Error::NotImplemented)
    }

    fn servo_config(&mut self, _: u8, _: Range<u16>) -> Result<(), Error> {
        Err(Error::NotImplemented)
    }

    fn i2c_config(&mut self, delay: u16) -> Result<(), Error> {
        self.protocol.i2c_config(delay)
    }

    fn i2c_read(&mut self, address: u8, size: u16) -> Result<(), Error> {
        self.protocol.i2c_read(address, size)
    }

    fn i2c_write(&mut self, address: u8, data: &[u16]) -> Result<(), Error> {
        self.protocol.i2c_write(address, data)
    }
}

impl Display for PCF8575 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let data = self.data.read();
        write!(
            f,
            "{} [address=0x{:02X}, firmware={}, version={}, protocol={}, transport=I2C]",
            self.get_name(),
            self.address,
            data.firmware_name,
            data.firmware_version,
            data.protocol_version,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::MockProtocol;

    #[test]
    fn test_helper() {
        let data = PCF8575::_build_pca9685_data();
        assert_eq!(data.firmware_name, "PCF8575");
        assert_eq!(data.protocol_version, "PCF8575");
        assert_eq!(data.pins.len(), 16);
    }

    #[test]
    fn test_default_initialization() {
        let board = Board::new(MockProtocol::default());
        let pcf8575 = PCF8575::default(&board).unwrap();

        assert_eq!(pcf8575.address, 0x20);
    }

    #[test]
    fn test_custom_initialization() {
        let board = Board::new(MockProtocol::default());
        let pcf8575 = PCF8575::new(&board, 0x41).unwrap();

        assert_eq!(pcf8575.address, 0x41);
    }

    #[test]
    fn test_set_pin_mode() {
        let board = Board::new(MockProtocol::default());
        let mut pcf8575 = PCF8575::default(&board).unwrap();

        // Test setting pin mode to OUTPUT
        assert!(pcf8575.set_pin_mode(0, PinModeId::OUTPUT).is_ok());

        // Test setting an incompatible pin mode
        let result = pcf8575.set_pin_mode(1, PinModeId::SERVO);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Hardware error: Pin (1) not compatible with mode (SERVO) - try to set pin mode."
        );

        // Test setting an invalid mode
        let result = pcf8575.set_pin_mode(2, PinModeId::DHT);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Hardware error: Pin (2) not compatible with mode (DHT) - try to set pin mode."
        );
    }

    #[test]
    fn test_digital_write() {
        let board = Board::new(MockProtocol::default());
        let mut pcf8575 = PCF8575::new(&board, 0x41).unwrap();

        let read = pcf8575.digital_write(1, true);
        assert!(
            read.is_ok(),
            "Digital write should not be an issue: {:?}",
            read
        );
        assert_eq!(pcf8575.data.read().get_pin(1).unwrap().value, 255); // pin 1 full ON
        assert_eq!(pcf8575.data.read().get_pin(2).unwrap().value, 0); // pin 2 unaffected

        assert!(pcf8575.digital_write(1, false).is_ok());
        assert_eq!(pcf8575.data.read().get_pin(1).unwrap().value, 0); // pin 1 full OFF
    }

    #[test]
    fn test_open() {
        let board = Board::new(MockProtocol::default());
        let mut pcf8575 = PCF8575::default(&board).unwrap();
        assert!(pcf8575.open().is_ok());
        assert!(pcf8575.is_connected());
    }

    #[test]
    fn test_close() {
        let board = Board::new(MockProtocol::default());
        let mut pcf8575 = PCF8575::default(&board).unwrap();
        pcf8575.data.write().connected = true; // force
        assert!(pcf8575.close().is_ok());
        assert!(!pcf8575.is_connected());
    }

    #[test]
    fn test_display_default() {
        let board = Board::new(MockProtocol::default());
        let pcf8575 = PCF8575::default(&board).unwrap();

        assert_eq!(
            format!("{}", pcf8575),
            "PCF8575 [address=0x20, firmware=PCF8575, version=n/a, protocol=PCF8575, transport=I2C]"
        );
    }

    #[test]
    fn test_hardware() {
        let board = Board::new(MockProtocol::default());
        let pcf8575 = PCF8575::new(&board, 0x41).unwrap();
        assert_eq!(
            pcf8575.get_protocol().to_string(),
            "PCF8575 [address=0x41, firmware=PCF8575, version=n/a, protocol=PCF8575, transport=I2C]"
        );
        assert_eq!(pcf8575.get_io().read().firmware_name, "PCF8575");
        assert!(pcf8575.is_connected());
    }
}
