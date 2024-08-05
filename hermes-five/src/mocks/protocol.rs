use std::sync::Arc;

use parking_lot::RwLock;

use crate::errors::Error;
use crate::mocks::hardware::create_test_hardware;
use crate::protocols::{Hardware, Protocol};

/// Mock implement for [`Protocol`].
/// Uses [`create_test_hardware`] for the hardware:
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct MockProtocol {
    pub connected: bool,
    /// The base-protocol attributes.
    #[cfg_attr(feature = "serde", serde(skip))]
    hardware: Arc<RwLock<Hardware>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub buf: [u8; 32],
    #[cfg_attr(feature = "serde", serde(skip))]
    pub index: usize,
}

impl Default for MockProtocol {
    fn default() -> Self {
        Self {
            connected: false,
            hardware: Arc::new(RwLock::new(create_test_hardware())),
            buf: [0; 32],
            index: 0,
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Protocol for MockProtocol {
    fn get_hardware(&self) -> &Arc<RwLock<Hardware>> {
        &self.hardware
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn set_connected(&mut self, status: bool) {
        self.connected = status;
    }

    fn open(&mut self) -> Result<(), Error> {
        self.connected = true;
        Ok(())
    }

    fn close(&mut self) -> Result<(), Error> {
        self.connected = false;
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        // Simulate write operation (for testing purposes)
        let len = self.buf.len().min(buf.len());
        self.buf[..len].copy_from_slice(&buf[..len]);
        Ok(())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        // Simulate read operation (for testing purposes)
        let len = self.buf.len().min(buf.len());
        buf[..len].copy_from_slice(&self.buf[self.index..self.index + len]);
        self.index += len;
        Ok(())
    }
}
