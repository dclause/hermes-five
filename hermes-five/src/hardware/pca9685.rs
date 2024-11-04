// ***********
// All information are relative to PCA9685 datasheets:
// https://www.digikey.jp/htmldatasheets/production/2459480/0/0/1/pca9685.html

use crate::errors::{Error, HardwareError, UnknownError};
use crate::hardware::{Board, Controller, Hardware};
use crate::io::{IoData, IoProtocol, Pin, PinMode, PinModeId, IO};
use crate::utils::{Range, Scalable};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct PCA9685 {
    // Address (default 0x40).
    address: u8,
    // Frequency in Mhz (default 50Mhz).
    frequency: u16,
    connected: bool,

    // ########################################
    // # Volatile utility data.
    #[cfg_attr(feature = "serde", serde(skip))]
    servo_configs: HashMap<u8, Range<u16>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    data: Arc<RwLock<IoData>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    protocol: Box<dyn IoProtocol>,
}

impl PCA9685 {
    // Registers.
    const MODE1: u8 = 0x0;
    // Magic bits.
    const PRESCALE: u8 = 0xFE;
    const BASE: u8 = 0x06;
    const SLEEP: u8 = 0x10;
    const RESET: u8 = 0x00;
    const RESTART: u8 = 0x80;
    const AUTO_INCREMENT: u8 = 0x20;
    // PCA9685 physical constraints.
    const MIN_FREQUENCY: u16 = 24; // Minimum frequency in Hz
    const MAX_FREQUENCY: u16 = 1526; // Maximum frequency in Hz
    const OSC_CLOCK: f32 = 25_000_000.0; // PCA9685 clock frequency

    fn _build_pca9685_data() -> IoData {
        let mut data = IoData {
            pins: Default::default(),
            i2c_data: vec![],
            digital_reported_pins: vec![],
            analog_reported_channels: vec![],
            protocol_version: "PCA9685".to_string(),
            firmware_name: "PCA9685".to_string(),
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
                            id: PinModeId::PWM,
                            resolution: 8,
                        },
                        PinMode {
                            id: PinModeId::SERVO,
                            resolution: 8,
                        },
                        PinMode {
                            id: PinModeId::ANALOG,
                            resolution: 8,
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
        PCA9685::new(board, 0x40)
    }

    pub fn new(board: &Board, address: u8) -> Result<Self, Error> {
        let protocol = board.get_protocol();
        let mut controller = Self {
            address,
            frequency: 50,
            connected: false,
            servo_configs: Default::default(),
            data: Arc::new(RwLock::new(PCA9685::_build_pca9685_data())),
            protocol,
        };
        IoProtocol::open(&mut controller)?;
        Ok(controller)
    }

    // Sets the PWM frequency (in Hz) for the entire PCA9685: from 24 to 1526 Hz.
    pub fn set_frequency(&mut self, frequency: u16) -> Result<&Self, Error> {
        // Validate frequency range
        if !(Self::MIN_FREQUENCY..=Self::MAX_FREQUENCY).contains(&frequency) {
            return Err(UnknownError {
                info: format!(
                    "Frequency must be between {} and {} Hz",
                    Self::MIN_FREQUENCY,
                    Self::MAX_FREQUENCY
                ),
            });
        };

        self.frequency = frequency;

        // 7.3.1 Mode register 1, MODE1 - Reset / Sleep
        // Sets the register mode to reset, than sleep.
        self.write_to_reg(PCA9685::MODE1, PCA9685::RESET)?;
        self.write_to_reg(PCA9685::MODE1, PCA9685::SLEEP)?;

        // 7.3.5 PWM frequency PRE_SCALE
        // prescale = round((osc_clock / (4096 x rate)) - 1) - with PCA9685 clock at 25Mhz
        // Calculate the prescale value for the desired frequency
        let prescale = ((PCA9685::OSC_CLOCK / (4096.0 * self.frequency as f32)) + 0.5 - 1.0)
            .clamp(3.0, 255.0) as u8;
        self.write_to_reg(PCA9685::PRESCALE, prescale)?;

        // Wake up and restart in auto-increment mode
        self.write_to_reg(PCA9685::MODE1, PCA9685::RESET)?;
        // std::thread::sleep(Duration::from_micros(5));
        self.write_to_reg(PCA9685::MODE1, PCA9685::RESTART | PCA9685::AUTO_INCREMENT)?;

        // trace!
        //     "Current i2c reply: 0x{:02X}",
        //     self.read_from_reg(PCA9685::MODE1)?
        // );
        // trace!
        //     "Current i2c prescale: {:#?}",
        //     self.read_from_reg(PCA9685::PRESCALE)?
        // );
        Ok(self)
    }

    pub fn write_to_reg(&mut self, register: u8, value: u8) -> Result<(), Error> {
        self.protocol
            .i2c_write(self.address, &[register as u16, value as u16])
    }

    pub fn read_from_reg(&mut self, register: u8) -> Result<u8, Error> {
        self.i2c_write(self.address, &[register as u16])?;
        self.i2c_read(self.address, 1)?;
        let register_value = {
            let lock = self.protocol.get_io().read();
            *lock.i2c_data.last().unwrap().data.last().unwrap()
        };
        Ok(register_value)
    }
}

impl Controller for PCA9685 {}

impl Hardware for PCA9685 {
    fn get_protocol(&self) -> Box<dyn IoProtocol> {
        Box::new(self.clone())
    }

    #[cfg(not(tarpaulin_include))]
    fn set_protocol(&mut self, protocol: Box<dyn IoProtocol>) {
        self.protocol = protocol;
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl IoProtocol for PCA9685 {
    fn open(&mut self) -> Result<(), Error> {
        self.i2c_config(0)?;
        self.connected = true;
        Ok(())
    }

    fn close(&mut self) -> Result<(), Error> {
        self.write_to_reg(PCA9685::MODE1, PCA9685::RESTART)?;
        self.connected = false;
        Ok(())
    }

    #[cfg(not(tarpaulin_include))]
    fn report_analog(&mut self, _: u8, _: bool) -> Result<(), Error> {
        unimplemented!();
    }

    #[cfg(not(tarpaulin_include))]
    fn report_digital(&mut self, _: u8, _: bool) -> Result<(), Error> {
        unimplemented!()
    }

    #[cfg(not(tarpaulin_include))]
    fn sampling_interval(&mut self, _: u16) -> Result<(), Error> {
        unimplemented!()
    }
}

impl IO for PCA9685 {
    fn get_io(&self) -> &Arc<RwLock<IoData>> {
        &self.data
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn set_pin_mode(&mut self, pin: u8, mode: PinModeId) -> Result<(), Error> {
        {
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
        }

        // Arbitrary selection of frequencies depending on pin mode.
        let frequency: u16 = match mode {
            PinModeId::OUTPUT => Ok(300), // Typical frequency to control a dimmable led.
            PinModeId::ANALOG => Ok(30),  // Typical frequency to control a fan.
            PinModeId::PWM => Ok(300),    // Typical frequency to control a dimmable led.
            PinModeId::SERVO => Ok(50),
            _ => Err(Error::from(HardwareError::IncompatiblePin {
                mode,
                pin,
                context: "update digital output",
            })),
        }?;
        self.set_frequency(frequency)?;

        Ok(())
    }

    fn digital_write(&mut self, pin: u8, level: bool) -> Result<(), Error> {
        let value = if level { 255 } else { 0 };
        self.analog_write(pin, value)
    }

    fn analog_write(&mut self, pin: u8, level: u16) -> Result<(), Error> {
        {
            let mut lock = self.data.write();
            // Check if pin exists
            let pin_instance = lock.get_pin_mut(pin)?;
            // Store the value we will write to the current pin.
            pin_instance.value = level;
        };

        // 7.3.3 LED output and PWM control
        // Creates a square signal on pin output.
        let servo_range = self.servo_configs.get(&pin);

        let (on, off): (u16, u16) = match servo_range {
            Some(_) => (0, (level as f32 / 4.88) as u16),
            None => {
                let level = level.clamp(0, 255);
                match level {
                    0 => (0, 4096),
                    255 => (4096, 0),
                    level => (0, level.scale(0, 255, 0, 4095)),
                }
            }
        };

        // The register corresponding to the pin (0-16) starts at BASE
        // see table 7 of the datasheet.
        let payload = &[(PCA9685::BASE + 4 * pin) as u16, on, on >> 8, off, off >> 8];

        // trace!(
        //     "I2C write: [on:{}, off:{}] {}",
        //     on,
        //     off,
        //     format_as_hex(payload)
        // );

        self.protocol.i2c_write(self.address, payload)
    }

    #[cfg(not(tarpaulin_include))]
    fn digital_read(&mut self, _: u8) -> Result<bool, Error> {
        unimplemented!()
    }

    #[cfg(not(tarpaulin_include))]
    fn analog_read(&mut self, _: u8) -> Result<u16, Error> {
        unimplemented!()
    }

    fn servo_config(&mut self, pin: u8, pwm_range: Range<u16>) -> Result<(), Error> {
        self.servo_configs.insert(pin, pwm_range);
        Ok(())
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
impl Display for PCA9685 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let data = self.data.read();
        write!(
            f,
            "{} [firmware={}, version={}, protocol={}, transport=I2C]",
            self.get_name(),
            data.firmware_name,
            data.firmware_version,
            data.protocol_version,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::RemoteIo;
    use crate::mocks::create_test_plugin_io_data;
    use crate::mocks::plugin_io::MockIoProtocol;
    use crate::mocks::transport_layer::MockTransportLayer;
    use crate::utils::Range;

    #[test]
    fn test_helper() {
        let data = PCA9685::_build_pca9685_data();
        assert_eq!(data.firmware_name, "PCA9685");
        assert_eq!(data.protocol_version, "PCA9685");
        assert_eq!(data.pins.len(), 16);
    }

    #[test]
    fn test_default_initialization() {
        let board = Board::new(MockIoProtocol::default());
        let pca9685 = PCA9685::default(&board).unwrap();

        assert_eq!(pca9685.address, 0x40);
        assert_eq!(pca9685.frequency, 50);
    }

    #[test]
    fn test_custom_initialization() {
        let board = Board::new(MockIoProtocol::default());
        let pca9685 = PCA9685::new(&board, 0x41).unwrap();

        assert_eq!(pca9685.address, 0x41);
        assert_eq!(pca9685.frequency, 50);
    }

    #[test]
    fn test_set_frequency_valid() {
        let board = Board::new(MockIoProtocol::default());
        let mut pca9685 = PCA9685::default(&board).unwrap();

        assert!(pca9685.set_frequency(100).is_ok());
        assert_eq!(pca9685.frequency, 100);
    }

    #[test]
    fn test_set_frequency_outofbound() {
        let board = Board::new(MockIoProtocol::default());
        let mut pca9685 = PCA9685::default(&board).unwrap();

        let result = pca9685.set_frequency(20);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Unknown error: Frequency must be between 24 and 1526 Hz."
        );

        let result = pca9685.set_frequency(1600);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Unknown error: Frequency must be between 24 and 1526 Hz."
        );
    }

    #[test]
    fn test_write_to_reg() {
        let transport = MockTransportLayer::default();
        let board = Board::new(RemoteIo::from(transport));
        let mut pca9685 = PCA9685::default(&board).unwrap();

        assert!(pca9685.write_to_reg(0x69, 0x42).is_ok());
    }

    #[test]
    fn test_read_from_reg() {
        let mut transport = MockTransportLayer::default();

        // Mock data for reading I2C reply of a single 0x69 register with value 0x42.
        let data = &[0xF0, 0x77, 0x40, 0x00, 0x69, 0x00, 0x42, 0x00, 0xF7];
        transport.read_buf[..data.len()].copy_from_slice(data);
        let protocol = RemoteIo::from(transport);
        *protocol.get_io().write() = create_test_plugin_io_data();

        let board = Board::new(protocol);
        let mut pca9685 = PCA9685::new(&board, 0x40).unwrap();

        let value = pca9685.read_from_reg(0x69).unwrap();
        assert_eq!(value, 0x42);
    }

    #[test]
    fn test_read_from_reg_failure() {
        let mut transport = MockTransportLayer::default();

        // Mock data for reading I2C reply too short.
        let data = &[0xF0, 0x77, 0x40, 0x00, 0xF7];
        transport.read_buf[..data.len()].copy_from_slice(data);
        let protocol = RemoteIo::from(transport);
        *protocol.get_io().write() = create_test_plugin_io_data();

        let board = Board::new(protocol);
        let mut pca9685 = PCA9685::new(&board, 0x40).unwrap();

        let result = pca9685.read_from_reg(PCA9685::MODE1);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_pin_mode() {
        let board = Board::new(MockIoProtocol::default());
        let mut pca9685 = PCA9685::default(&board).unwrap();

        // Test setting pin mode to OUTPUT
        assert!(pca9685.set_pin_mode(0, PinModeId::OUTPUT).is_ok());
        assert_eq!(pca9685.frequency, 300);

        // Test setting pin mode to SERVO
        assert!(pca9685.set_pin_mode(1, PinModeId::SERVO).is_ok());
        assert_eq!(pca9685.frequency, 50);

        // Test setting pin mode to ANALOG
        assert!(pca9685.set_pin_mode(1, PinModeId::ANALOG).is_ok());
        assert_eq!(pca9685.frequency, 30);

        // Test setting pin mode to PWM
        assert!(pca9685.set_pin_mode(1, PinModeId::PWM).is_ok());
        assert_eq!(pca9685.frequency, 300);

        // Test setting an invalid mode
        let result = pca9685.set_pin_mode(2, PinModeId::UNSUPPORTED);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Hardware error: Pin (2) not compatible with mode (UNSUPPORTED) - try to set pin mode."
        )
    }

    #[test]
    fn test_digital_write() {
        let board = Board::new(MockIoProtocol::default());
        let mut pca9685 = PCA9685::new(&board, 0x41).unwrap();

        assert!(pca9685.digital_write(1, true).is_ok());
        let value = pca9685.data.read().get_pin(1).unwrap().value;
        assert_eq!(value, 255);

        assert!(pca9685.digital_write(1, false).is_ok());
        let value = pca9685.data.read().get_pin(1).unwrap().value;
        assert_eq!(value, 0);
    }

    #[test]
    fn test_analog_write() {
        let board = Board::new(MockIoProtocol::default());
        let mut pca9685 = PCA9685::default(&board).unwrap();

        assert!(pca9685.analog_write(0, 128).is_ok());
        let value = pca9685.data.read().get_pin(0).unwrap().value;
        assert_eq!(value, 128);

        assert!(pca9685.analog_write(0, 0).is_ok());
        let value = pca9685.data.read().get_pin(0).unwrap().value;
        assert_eq!(value, 0);

        assert!(pca9685.analog_write(0, 255).is_ok());
        let value = pca9685.data.read().get_pin(0).unwrap().value;
        assert_eq!(value, 255);

        pca9685.data.write().get_pin_mut(1).unwrap().mode.id = PinModeId::SERVO;
        assert!(pca9685.servo_config(1, Range::from([300, 600])).is_ok());
        assert!(pca9685.analog_write(1, 128).is_ok());
        let value = pca9685.data.read().get_pin(1).unwrap().value;
        assert_eq!(value, 128);
    }

    #[test]
    fn test_servo_config() {
        let board = Board::new(MockIoProtocol::default());
        let mut pca9685 = PCA9685::default(&board).unwrap();

        // Test configuring the servo
        let pwm_range = Range::from([1000, 2000]);
        assert!(pca9685.servo_config(0, pwm_range).is_ok());

        // Verify servo config
        assert!(pca9685.servo_configs.contains_key(&0));
        assert_eq!(pca9685.servo_configs.get(&0).unwrap().start, 1000);
        assert_eq!(pca9685.servo_configs.get(&0).unwrap().end, 2000);
    }

    #[test]
    fn test_open() {
        let board = Board::new(MockIoProtocol::default());
        let mut pca9685 = PCA9685::default(&board).unwrap();
        assert!(pca9685.open().is_ok());
        assert!(pca9685.is_connected());
    }

    #[test]
    fn test_close() {
        let board = Board::new(MockIoProtocol::default());
        let mut pca9685 = PCA9685::default(&board).unwrap();
        pca9685.connected = true; // force
        assert!(pca9685.close().is_ok());
        assert!(!pca9685.is_connected());
    }

    #[test]
    fn test_display() {
        let board = Board::new(MockIoProtocol::default());
        let pca9685 = PCA9685::default(&board).unwrap();

        assert_eq!(
            format!("{}", pca9685),
            "PCA9685 [firmware=PCA9685, version=n/a, protocol=PCA9685, transport=I2C]"
        );
    }

    #[test]
    fn test_hardware() {
        let board = Board::new(MockIoProtocol::default());
        let pca9685 = PCA9685::new(&board, 0x41).unwrap();
        assert_eq!(
            pca9685.get_protocol().to_string(),
            "PCA9685 [firmware=PCA9685, version=n/a, protocol=PCA9685, transport=I2C]"
        );
        assert_eq!(pca9685.get_io().read().firmware_name, "PCA9685");
        assert!(pca9685.is_connected());
    }
}
