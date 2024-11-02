use crate::errors::Error;
use crate::errors::HardwareError::IncompatibleMode;
use crate::io::{IoData, IoProtocol, PinModeId, IO};
use crate::mocks::create_test_plugin_io_data;
use crate::pause_sync;
use crate::utils::Range;
use parking_lot::RwLock;
use std::fmt::Display;
use std::sync::Arc;

/// Mock implement for [`IoData`].
/// Uses [`create_test_plugin_io_data`] for the hardware:
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct MockIoProtocol {
    #[cfg_attr(feature = "serde", serde(skip))]
    pub connected: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub data: Arc<RwLock<IoData>>,
}

impl Default for MockIoProtocol {
    fn default() -> Self {
        Self {
            connected: false,
            data: Arc::new(RwLock::new(create_test_plugin_io_data())),
        }
    }
}

impl Display for MockIoProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.data.read();
        write!(
            f,
            "{} [firmware={}, version={}, protocol={}]",
            self.get_name(),
            data.firmware_name,
            data.firmware_version,
            data.protocol_version,
        )
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl IoProtocol for MockIoProtocol {
    fn open(&mut self) -> Result<(), Error> {
        pause_sync!(100);
        self.connected = true;
        Ok(())
    }

    fn close(&mut self) -> Result<(), Error> {
        pause_sync!(100);
        self.connected = false;
        Ok(())
    }

    fn report_analog(&mut self, _: u8, _: bool) -> Result<(), Error> {
        Ok(())
    }

    fn report_digital(&mut self, _: u8, _: bool) -> Result<(), Error> {
        Ok(())
    }

    fn sampling_interval(&mut self, _: u16) -> Result<(), Error> {
        Ok(())
    }
}

impl IO for MockIoProtocol {
    fn get_io(&self) -> &Arc<RwLock<IoData>> {
        &self.data
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn set_pin_mode(&mut self, pin: u8, mode: PinModeId) -> Result<(), Error> {
        let mut lock = self.data.write();
        let pin_instance = lock.get_pin_mut(pin)?;
        let _mode = pin_instance.supports_mode(mode).ok_or(IncompatibleMode {
            pin,
            mode,
            context: "try to set pin mode",
        })?;
        pin_instance.mode = _mode;
        Ok(())
    }

    fn digital_write(&mut self, pin: u8, level: bool) -> Result<(), Error> {
        let mut lock = self.data.write();
        let pin_instance = lock.get_pin_mut(pin)?;
        pin_instance.validate_current_mode(PinModeId::OUTPUT)?;
        pin_instance.value = u16::from(level);
        Ok(())
    }

    fn analog_write(&mut self, pin: u8, level: u16) -> Result<(), Error> {
        self.data.write().get_pin_mut(pin)?.value = level;
        Ok(())
    }

    fn digital_read(&mut self, _: u8) -> Result<bool, Error> {
        unimplemented!()
    }

    fn analog_read(&mut self, _: u8) -> Result<u16, Error> {
        unimplemented!()
    }

    fn servo_config(&mut self, _: u8, _: Range<u16>) -> Result<(), Error> {
        Ok(())
    }

    fn i2c_config(&mut self, _: u16) -> Result<(), Error> {
        Ok(())
    }

    fn i2c_read(&mut self, _: u8, _: u16) -> Result<(), Error> {
        Ok(())
    }

    fn i2c_write(&mut self, _: u8, _: &[u16]) -> Result<(), Error> {
        Ok(())
    }
}
