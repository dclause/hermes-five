use crate::errors::Error;
use crate::errors::HardwareError::IncompatiblePin;
use crate::io::{IoData, IoProtocol, PinModeId, IO};
use crate::mocks::create_test_plugin_io_data;
use crate::pause_sync;
use crate::utils::Range;
use parking_lot::RwLock;
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
    pub data: Arc<RwLock<IoData>>,
}

impl Default for MockProtocol {
    fn default() -> Self {
        Self {
            connected: Arc::new(AtomicBool::new(false)),
            data: Arc::new(RwLock::new(create_test_plugin_io_data())),
        }
    }
}

impl Display for MockProtocol {
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

impl IO for MockProtocol {
    fn get_io(&self) -> &Arc<RwLock<IoData>> {
        &self.data
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    fn set_pin_mode(&self, pin: u8, mode: PinModeId) -> Result<(), Error> {
        let mut lock = self.data.write();
        let pin_instance = lock.get_pin_mut(pin)?;
        let _mode = pin_instance.supports_mode(mode).ok_or(IncompatiblePin {
            pin,
            mode,
            context: "try to set pin mode",
        })?;
        pin_instance.mode = _mode;
        Ok(())
    }

    fn digital_write(&self, pin: u8, level: bool) -> Result<(), Error> {
        let mut lock = self.data.write();
        let pin_instance = lock.get_pin_mut(pin)?;
        pin_instance.validate_current_mode(PinModeId::OUTPUT)?;
        pin_instance.value = u16::from(level);
        Ok(())
    }

    fn analog_write(&self, pin: u8, level: u16) -> Result<(), Error> {
        self.data.write().get_pin_mut(pin)?.value = level;
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
}
