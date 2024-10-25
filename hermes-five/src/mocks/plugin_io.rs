use crate::errors::Error;
use crate::errors::HardwareError::IncompatibleMode;
use crate::io::{PinModeId, PluginIO, PluginIoData};
use crate::mocks::create_test_plugin_io_data;
use crate::pause_sync;
use crate::utils::Range;
use parking_lot::RwLock;
use std::fmt::Display;
use std::sync::Arc;

/// Mock implement for [`PluginIoData`].
/// Uses [`create_test_plugin_io_data`] for the hardware:
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct MockPluginIO {
    pub connected: bool,
    // pub inner: FirmataIO,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub data: Arc<RwLock<PluginIoData>>,
}

impl Default for MockPluginIO {
    fn default() -> Self {
        Self {
            connected: false,
            // inner: FirmataIO::from(MockTransportLayer::default()),
            data: Arc::new(RwLock::new(create_test_plugin_io_data())),
        }
    }
}

impl Display for MockPluginIO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.data.read();
        write!(
            f,
            "{} [firmware={}, version={}, protocol={}]",
            self.get_protocol_name(),
            data.firmware_name,
            data.firmware_version,
            data.protocol_version,
        )
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl PluginIO for MockPluginIO {
    fn get_data(&self) -> &Arc<RwLock<PluginIoData>> {
        &self.data
    }

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

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn set_pin_mode(&mut self, pin: u16, mode: PinModeId) -> Result<(), Error> {
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

    fn digital_write(&mut self, pin: u16, level: bool) -> Result<(), Error> {
        let mut lock = self.data.write();
        let pin_instance = lock.get_pin_mut(pin)?;
        pin_instance.validate_current_mode(PinModeId::OUTPUT)?;
        pin_instance.value = u16::from(level);
        Ok(())
    }

    fn analog_write(&mut self, pin: u16, level: u16) -> Result<(), Error> {
        self.data.write().get_pin_mut(pin)?.value = level;
        Ok(())
    }

    fn report_analog(&mut self, _: u8, _: bool) -> Result<(), Error> {
        Ok(())
    }

    fn report_digital(&mut self, _: u16, _: bool) -> Result<(), Error> {
        Ok(())
    }

    fn sampling_interval(&mut self, _: u16) -> Result<(), Error> {
        Ok(())
    }

    fn i2c_config(&mut self, _: u16) -> Result<(), Error> {
        Ok(())
    }

    fn i2c_read(&mut self, _: i32, _: i32) -> Result<(), Error> {
        Ok(())
    }

    fn i2c_write(&mut self, _: i32, _: &[u8]) -> Result<(), Error> {
        Ok(())
    }

    fn servo_config(&mut self, _: u16, _: Range<u16>) -> Result<(), Error> {
        Ok(())
    }
}
