use crate::errors::Error;
use crate::hardware::{I2CReply, LowLevelApi, LowLevelApiExt, Pin, PinModeId};
use crate::mocks::create_test_pins;
use crate::pause_sync;
use crate::protocols::IoProtocol;
use crate::utils::Range;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Mock implement for [`IoData`].
/// Uses [`create_test_plugin_io_data`] for the hardware:
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct MockProtocol {
    #[cfg_attr(feature = "serde", serde(skip))]
    pub connected: Arc<AtomicBool>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pins: HashMap<u8, Arc<Pin>>,
}

impl Default for MockProtocol {
    fn default() -> Self {
        Self {
            connected: Arc::new(AtomicBool::new(false)),
            pins: create_test_pins(),
        }
    }
}

impl Display for MockProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MockProtocol [protocol={} (version={}), firmware={} (version {})]",
            self.get_protocol_name(),
            self.get_protocol_version(),
            self.get_firmware_name(),
            self.get_firmware_version(),
        )
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl IoProtocol for MockProtocol {
    fn open(&self) -> Result<(), Error> {
        pause_sync!(100);
        self.connected.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn close(&self) -> Result<(), Error> {
        pause_sync!(100);
        self.connected.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn report_analog(&self, _: u8, _: bool) -> Result<(), Error> {
        Ok(())
    }

    fn report_digital(&self, _: u8, _: bool) -> Result<(), Error> {
        Ok(())
    }

    fn sampling_interval(&self, _: u16) -> Result<(), Error> {
        Ok(())
    }
}

impl LowLevelApi for MockProtocol {
    fn get_protocol_name(&self) -> &str {
        "MockProtocol"
    }

    fn get_protocol_version(&self) -> &str {
        "fake.2.3"
    }

    fn get_firmware_name(&self) -> &str {
        "fake_firmware"
    }

    fn get_firmware_version(&self) -> &str {
        "fake.1.0"
    }

    fn get_pins(&self) -> &HashMap<u8, Arc<Pin>> {
        &self.pins
    }

    fn set_pin_mode(&self, pin: u8, mode: PinModeId) -> Result<(), Error> {
        self.get_pin(pin)?.set_pin_mode(mode)?;
        Ok(())
    }

    fn digital_write(&self, pin: u8, level: bool) -> Result<(), Error> {
        let pin_instance = self.get_pin(pin)?;
        pin_instance.ensure_mode_is(PinModeId::OUTPUT)?;
        pin_instance.set_value(if level { u16::MAX } else { u16::MIN });
        Ok(())
    }

    fn analog_write(&self, pin: u8, level: u16) -> Result<(), Error> {
        let pin_instance = self.get_pin(pin)?;
        pin_instance.set_value(level);
        Ok(())
    }

    fn digital_read(&self, _: u8) -> Result<bool, Error> {
        Err(Error::NotImplemented)
    }

    fn analog_read(&self, _: u8) -> Result<u16, Error> {
        Err(Error::NotImplemented)
    }

    fn servo_config(&self, _: u8, _: Range<u16>) -> Result<(), Error> {
        Ok(())
    }

    fn i2c_config(&self, _: u16) -> Result<(), Error> {
        Ok(())
    }

    fn i2c_read(&self, _: u8, _: u16) -> Result<(), Error> {
        Ok(())
    }

    fn i2c_write(&self, _: u8, _: &[u16]) -> Result<(), Error> {
        Ok(())
    }

    fn get_i2c_data(&self, _: u8) -> Arc<RwLock<Vec<I2CReply>>> {
        todo!()
    }
}
