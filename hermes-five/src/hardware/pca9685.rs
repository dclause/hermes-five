// ***********
// All information are relative to PCA9685 datasheets:
// https://www.digikey.jp/htmldatasheets/production/2459480/0/0/1/pca9685.html

use crate::errors::{Error, HardwareError, InternalError};
use crate::hardware::{
    Board, Hardware, I2CReply, LowLevelApi, LowLevelApiExt, Pin, PinMode, PinModeId,
};
use crate::protocols::IoProtocol;
use crate::utils::{Range, Scalable};
use hermes_five_macros::Expander;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, OnceLock};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Expander)]
pub struct PCA9685 {
    // Address (default 0x40).
    address: u8,
    // Frequency in Mhz (default 50Mhz).
    #[cfg_attr(feature = "serde", serde(with = "crate::utils::serde_arc_atomic"))]
    frequency: Arc<AtomicU16>,

    // ########################################
    // # Volatile utility data.
    #[cfg_attr(feature = "serde", serde(skip))]
    pins: Arc<OnceLock<HashMap<u8, Arc<Pin>>>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    servo_configs: Arc<RwLock<HashMap<u8, Range<u16>>>>,
    #[cfg_attr(
        feature = "serde",
        serde(with = "crate::utils::serde_arc_protocol", skip_serializing)
    )]
    protocol: Arc<dyn IoProtocol>,
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

    fn _build_pca9685_pins() -> HashMap<u8, Arc<Pin>> {
        let mut pins = HashMap::new();
        for id in 0..16 {
            pins.insert(
                id,
                Arc::new(Pin {
                    id,
                    name: Arc::new(OnceLock::from(format!("D{}", id))),
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
                        PinMode {
                            id: PinModeId::UNSUPPORTED,
                            resolution: 0,
                        },
                    ],
                    channel: Arc::new(OnceLock::from(None)),
                    value: Default::default(),
                }),
            );
        }
        pins
    }

    pub fn default(board: &Board) -> Result<Self, Error> {
        PCA9685::new(board, 0x40)
    }

    pub fn new(board: &dyn Hardware, address: u8) -> Result<Self, Error> {
        let mut expander = Self {
            address,
            frequency: Arc::new(AtomicU16::new(50)),
            pins: Arc::new(OnceLock::from(Self::_build_pca9685_pins())),
            servo_configs: Default::default(),
            protocol: board.get_protocol(),
        };
        IoProtocol::open(&mut expander)?;
        Ok(expander)
    }

    pub fn get_address(&self) -> u8 {
        self.address
    }

    pub fn get_frequency(&self) -> u16 {
        self.frequency.load(Ordering::Relaxed)
    }

    // Sets the PWM frequency (in Hz) for the entire PCA9685: from 24 to 1526 Hz.
    pub fn set_frequency(&self, frequency: u16) -> Result<&Self, Error> {
        // Validate frequency range
        if !(Self::MIN_FREQUENCY..=Self::MAX_FREQUENCY).contains(&frequency) {
            return Err(InternalError {
                info: format!(
                    "Frequency must be between {} and {} Hz",
                    Self::MIN_FREQUENCY,
                    Self::MAX_FREQUENCY
                ),
            });
        };

        self.frequency.store(frequency, Ordering::Relaxed);

        // 7.3.1 Mode register 1, MODE1 - Reset / Sleep
        // Sets the register mode to reset, than sleep.
        self.write_to_reg(PCA9685::MODE1, PCA9685::RESET)?;
        self.write_to_reg(PCA9685::MODE1, PCA9685::SLEEP)?;

        // 7.3.5 PWM frequency PRE_SCALE
        // prescale = round((osc_clock / (4096 x rate)) - 1) - with PCA9685 clock at 25Mhz
        // Calculate the prescale value for the desired frequency
        let prescale = ((PCA9685::OSC_CLOCK / (4096.0 * frequency as f32)) + 0.5 - 1.0)
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

    pub fn write_to_reg(&self, register: u8, value: u8) -> Result<(), Error> {
        self.protocol
            .i2c_write(self.address, &[register as u16, value as u16])
    }

    pub fn read_from_reg(&self, register: u8) -> Result<u8, Error> {
        self.i2c_write(self.address, &[register as u16])?;
        self.i2c_read(self.address, 1)?;
        todo!("Implement reading i2C");
        // let register_value = {
        //     let lock = self.protocol.get_io().read();
        //     *lock.i2c_data.last().unwrap().data.last().unwrap()
        // };
        // Ok(register_value)
    }
}

impl Hardware for PCA9685 {
    fn get_protocol(&self) -> Arc<dyn IoProtocol> {
        Arc::new(self.clone())
    }

    #[cfg(feature = "serde")]
    fn set_protocol(&mut self, protocol: Arc<dyn IoProtocol>) {
        self.protocol = protocol;
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl IoProtocol for PCA9685 {
    fn open(&self) -> Result<(), Error> {
        self.i2c_config(0)?;
        Ok(())
    }

    fn close(&self) -> Result<(), Error> {
        self.write_to_reg(PCA9685::MODE1, PCA9685::RESTART)?;
        Ok(())
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn report_analog(&self, _: u8, _: bool) -> Result<(), Error> {
        Err(Error::NotImplemented)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn report_digital(&self, _: u8, _: bool) -> Result<(), Error> {
        Err(Error::NotImplemented)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn sampling_interval(&self, _: u16) -> Result<(), Error> {
        Err(Error::NotImplemented)
    }
}

impl LowLevelApi for PCA9685 {
    fn get_protocol_name(&self) -> &str {
        "PCA9685"
    }

    fn get_protocol_version(&self) -> &str {
        "N/A"
    }

    fn get_firmware_name(&self) -> &str {
        "N/A"
    }

    fn get_firmware_version(&self) -> &str {
        "N/A"
    }

    fn get_pins(&self) -> &HashMap<u8, Arc<Pin>> {
        self.pins.get().unwrap()
    }

    fn set_pin_mode(&self, pin: u8, mode: PinModeId) -> Result<(), Error> {
        let pin_instance = self.get_pin(pin)?;
        pin_instance.set_pin_mode(mode)?;

        // Special hack: unsupported should disable the pin, hence send no signal at all.
        if mode == PinModeId::UNSUPPORTED {
            let payload = &[(PCA9685::BASE + 4 * pin) as u16, 0, 0, 4096, 4096 >> 8];
            self.protocol.i2c_write(self.address, payload)?;
            return Ok(());
        }

        // Arbitrary selection of frequencies depending on pin mode.
        let frequency: u16 = match mode {
            PinModeId::OUTPUT => Ok(300), // Typical frequency to control a dimmable led.
            PinModeId::ANALOG => Ok(30),  // Typical frequency to control a fan.
            PinModeId::PWM => Ok(300),    // Typical frequency to control a dimmable led.
            PinModeId::SERVO => Ok(50),
            _ => Err(Error::from(HardwareError::IncompatiblePin { mode, pin })),
        }?;
        self.set_frequency(frequency)?;

        Ok(())
    }

    fn digital_write(&self, pin: u8, level: bool) -> Result<(), Error> {
        let value = if level { 0xFF } else { 0x00 };
        self.analog_write(pin, value)
    }

    fn analog_write(&self, pin: u8, level: u16) -> Result<(), Error> {
        let level = level.clamp(0, 255);

        let pin_instance = self.get_pin(pin)?;
        pin_instance.set_value(level);

        // 7.3.3 LED output and PWM control
        // Creates a square signal on pin output.

        // - The 'ON' impulsion is always triggered at t=0
        // - The 'OFF' impulsion is proportional to `level`
        // Note: On PCA9685, a period is divided in 4096 ticks (12 bits).
        let (on, off): (u16, u16) = match self.servo_configs.read().get(&pin) {
            // Servo range = SERVO control.
            Some(range) => {
                let pwm = level.scale(0, 0xFF, range.start, range.end);
                (0, pwm)
            }
            // No servo ranges = PWM control.
            None => {
                let level = level.clamp(0, 255);
                match level {
                    0 => (0, 4096),    // completely OFF
                    0xFF => (4096, 0), // completely ON
                    level => (0, level.scale(0, 0xFF, 0, 4096)),
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

        self.protocol.i2c_write(self.address, payload)?;
        Ok(())
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn digital_read(&self, _: u8) -> Result<bool, Error> {
        Err(Error::NotImplemented)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn analog_read(&self, _: u8) -> Result<u16, Error> {
        Err(Error::NotImplemented)
    }

    fn servo_config(&self, pin: u8, pwm_range: Range<u16>) -> Result<(), Error> {
        self.servo_configs.write().insert(pin, pwm_range);
        Ok(())
    }

    fn i2c_config(&self, delay: u16) -> Result<(), Error> {
        self.protocol.i2c_config(delay)
    }

    fn i2c_read(&self, address: u8, size: u16) -> Result<(), Error> {
        self.protocol.i2c_read(address, size)
    }

    fn i2c_write(&self, address: u8, data: &[u16]) -> Result<(), Error> {
        self.protocol.i2c_write(address, data)
    }

    fn get_i2c_data(&self, _: u8) -> Arc<RwLock<Vec<I2CReply>>> {
        todo!("to be implemented")
    }
}
impl Display for PCA9685 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PCA9685 [address=0x{:02X}, protocol={} (version={}), firmware={} (version {}), transport=I2C]",
            self.address,
            self.get_protocol_name(),
            self.get_protocol_version(),
            self.get_firmware_name(),
            self.get_firmware_version(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::MockProtocol;
    use crate::mocks::MockTransport;
    use crate::protocols::RemoteIo;
    use crate::utils::Range;

    #[test]
    fn test_default_initialization() {
        let board = Board::new(MockProtocol::default());
        let pca9685 = PCA9685::default(&board).unwrap();

        assert_eq!(pca9685.address, 0x40);
        assert_eq!(pca9685.frequency.load(Ordering::Relaxed), 50);
    }

    #[test]
    fn test_custom_initialization() {
        let board = Board::new(MockProtocol::default());
        let pca9685 = PCA9685::new(&board, 0x41).unwrap();

        assert_eq!(pca9685.address, 0x41);
        assert_eq!(pca9685.frequency.load(Ordering::Relaxed), 50);
    }

    #[test]
    fn test_set_frequency_valid() {
        let board = Board::new(MockProtocol::default());
        let pca9685 = PCA9685::default(&board).unwrap();

        assert!(pca9685.set_frequency(100).is_ok());
        assert_eq!(pca9685.frequency.load(Ordering::Relaxed), 100);
    }

    #[test]
    fn test_set_frequency_outofbound() {
        let board = Board::new(MockProtocol::default());
        let pca9685 = PCA9685::default(&board).unwrap();

        let result = pca9685.set_frequency(20);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Internal error: Frequency must be between 24 and 1526 Hz."
        );

        let result = pca9685.set_frequency(1600);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Internal error: Frequency must be between 24 and 1526 Hz."
        );
    }

    #[test]
    fn test_write_to_reg() {
        let transport = MockTransport::default();
        let board = Board::new(RemoteIo::from(transport));
        let pca9685 = PCA9685::default(&board).unwrap();

        assert!(pca9685.write_to_reg(0x69, 0x42).is_ok());
    }

    // #[test]
    // fn test_read_from_reg() {
    //     // Mock data for reading I2C reply of a single 0x69 register with value 0x42.
    //     let data = &[0xF0, 0x77, 0x40, 0x00, 0x69, 0x00, 0x42, 0x00, 0xF7];
    //
    //     let transport = MockTransport::new(data.to_vec());
    //     let protocol = RemoteIo::from(transport);
    //     let board = Board::new(protocol);
    //     let pca9685 = PCA9685::new(&board, 0x40).unwrap();
    //
    //     let value = pca9685.read_from_reg(0x69).unwrap();
    //     assert_eq!(value, 0x42);
    // }

    #[test]
    fn test_read_from_reg_failure() {
        // Mock data for reading I2C reply too short.
        let data = &[0xF0, 0x77, 0x40, 0x00, 0xF7];

        let transport = MockTransport::new(data.to_vec());
        let protocol = RemoteIo::from(transport);

        let board = Board::new(protocol);
        let pca9685 = PCA9685::new(&board, 0x40).unwrap();

        let result = pca9685.read_from_reg(PCA9685::MODE1);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_pin_mode() {
        let board = Board::new(MockProtocol::default());
        let pca9685 = PCA9685::default(&board).unwrap();

        // Test setting pin mode to OUTPUT
        assert!(pca9685.set_pin_mode(0, PinModeId::OUTPUT).is_ok());
        assert_eq!(pca9685.get_frequency(), 300);

        // Test setting pin mode to SERVO
        assert!(pca9685.set_pin_mode(1, PinModeId::SERVO).is_ok());
        assert_eq!(pca9685.get_frequency(), 50);

        // Test setting pin mode to ANALOG
        assert!(pca9685.set_pin_mode(1, PinModeId::ANALOG).is_ok());
        assert_eq!(pca9685.get_frequency(), 30);

        // Test setting pin mode to PWM
        assert!(pca9685.set_pin_mode(1, PinModeId::PWM).is_ok());
        assert_eq!(pca9685.get_frequency(), 300);

        // Test setting an invalid mode
        let result = pca9685.set_pin_mode(2, PinModeId::DHT);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Hardware error: Pin (2) not compatible with mode (DHT)."
        )
    }

    #[test]
    fn test_digital_write() {
        let board = Board::new(MockProtocol::default());
        let pca9685 = PCA9685::new(&board, 0x41).unwrap();

        assert!(pca9685.digital_write(1, true).is_ok());
        assert_eq!(pca9685.get_pin(1).unwrap().get_value(), 255);

        assert!(pca9685.digital_write(1, false).is_ok());
        assert_eq!(pca9685.get_pin(1).unwrap().get_value(), 0);
    }

    #[test]
    fn test_analog_write() {
        let board = Board::new(MockProtocol::default());
        let pca9685 = PCA9685::default(&board).unwrap();

        assert!(pca9685.analog_write(1, 128).is_ok());
        assert_eq!(pca9685.get_pin(1).unwrap().get_value(), 128);

        assert!(pca9685.analog_write(1, 0).is_ok());
        assert_eq!(pca9685.get_pin(1).unwrap().get_value(), 0);

        assert!(pca9685.analog_write(1, 255).is_ok());
        assert_eq!(pca9685.get_pin(1).unwrap().get_value(), 0xFF);

        pca9685.set_pin_mode(1, PinModeId::SERVO).unwrap();
        assert!(pca9685.servo_config(1, Range::from([300, 600])).is_ok());
        assert!(pca9685.analog_write(1, 128).is_ok());
        assert_eq!(pca9685.get_pin(1).unwrap().get_value(), 128);
    }

    #[test]
    fn test_servo_config() {
        let board = Board::new(MockProtocol::default());
        let pca9685 = PCA9685::default(&board).unwrap();

        // Test configuring the servo
        let pwm_range = Range::from([1000, 2000]);
        assert!(pca9685.servo_config(0, pwm_range).is_ok());

        // Verify servo config
        let servo_configs = pca9685.servo_configs.read();
        assert!(servo_configs.contains_key(&0));
        assert_eq!(servo_configs.get(&0).unwrap().start, 1000);
        assert_eq!(servo_configs.get(&0).unwrap().end, 2000);
    }

    #[test]
    fn test_open() {
        let board = Board::new(MockProtocol::default());
        let pca9685 = PCA9685::default(&board).unwrap();
        assert!(pca9685.open().is_ok());
    }

    #[test]
    fn test_close() {
        let board = Board::new(MockProtocol::default());
        let pca9685 = PCA9685::default(&board).unwrap();
        assert!(pca9685.close().is_ok());
    }

    #[test]
    fn test_display() {
        let board = Board::new(MockProtocol::default());
        let pca9685 = PCA9685::default(&board).unwrap();

        assert_eq!(
            format!("{}", pca9685),
            "PCA9685 [address=0x40, protocol=PCA9685 (version=N/A), firmware=N/A (version N/A), transport=I2C]"
        );
    }

    #[test]
    fn test_hardware() {
        let board = Board::new(MockProtocol::default());
        let pca9685 = PCA9685::new(&board, 0x41).unwrap();
        assert_eq!(
            pca9685.get_protocol().to_string(),
            "PCA9685 [address=0x41, protocol=PCA9685 (version=N/A), firmware=N/A (version N/A), transport=I2C]"
        );
        assert_eq!(pca9685.get_protocol_name(), "PCA9685");
        assert_eq!(pca9685.get_protocol_version(), "N/A");
        assert_eq!(pca9685.get_firmware_version(), "N/A");
        assert_eq!(pca9685.get_firmware_name(), "N/A");
    }
}
