use crate::errors::Error;
use crate::io::TransportLayer;
use crate::pause_sync;
use std::fmt::{Display, Formatter};
use std::time::Duration;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default)]
pub struct MockTransportLayer {
    pub connected: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub read_buf: [u8; 32],
    #[cfg_attr(feature = "serde", serde(skip))]
    pub write_buf: [u8; 32],
    #[cfg_attr(feature = "serde", serde(skip))]
    pub read_index: usize,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub write_index: usize,
}

impl Display for MockTransportLayer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockTransportLayer")
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl TransportLayer for MockTransportLayer {
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

    fn set_timeout(&mut self, _: Duration) -> Result<(), Error> {
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        // Simulate write operation (for testing purposes)
        let len = self.write_buf.len().min(buf.len());
        self.write_buf[self.write_index..self.write_index + len].copy_from_slice(&buf[..len]);
        if self.write_index + len > self.write_buf.len() {
            self.write_index = 0;
        } else {
            self.write_index += len;
        }
        Ok(())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        // Simulate read operation (for testing purposes)
        let len = self.read_buf.len().min(buf.len());
        // Loop over.
        if self.read_index + len > self.read_buf.len() {
            self.read_index = 0;
        }
        buf[..len].copy_from_slice(&self.read_buf[self.read_index..self.read_index + len]);
        self.read_index += len;
        Ok(())
    }
}
