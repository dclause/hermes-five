use crate::errors::Error;
use crate::pause_sync;
use crate::transports::IoTransport;
use parking_lot::RwLock;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct MockTransport {
    #[cfg_attr(feature = "serde", serde(skip))]
    pub connected: Arc<AtomicBool>,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub read_buf: Arc<RwLock<Vec<u8>>>,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub write_buf: Arc<RwLock<Vec<u8>>>,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub read_index: Arc<RwLock<usize>>,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub write_index: Arc<RwLock<usize>>,
}

impl Default for MockTransport {
    fn default() -> Self {
        Self {
            connected: Arc::new(Default::default()),
            read_buf: Arc::new(RwLock::new(vec![
                0xF0, 0x79, 0x01, 0x0C, 0xF7, // Result for query firmware
                0xF0, 0x6C, // Start result for report capabilities
                0x7F, 0x7F, 0x00, 0x01, 0x0B, 0x01, 0x01, 0x01, 0x04, 0x0E, 0x7F, 0x00, 0x01, 0x0B,
                0x01, 0x01, 0x01, 0x03, 0x08, 0x04, 0x0E, 0x7F, 0x00, 0x01, 0x0B, 0x01, 0x01, 0x01,
                0x04, 0x0E, 0x7F, 0x00, 0x01, 0x0B, 0x01, 0x01, 0x01, 0x03, 0x08, 0x04, 0x0E, 0x7F,
                0x00, 0x01, 0x0B, 0x01, 0x01, 0x01, 0x03, 0x08, 0x04, 0x0E, 0x7F, 0x00, 0x01, 0x0B,
                0x01, 0x01, 0x01, 0x04, 0x0E, 0x7F, 0x00, 0x01, 0x0B, 0x01, 0x01, 0x01, 0x04, 0x0E,
                0x7F, 0x00, 0x01, 0x0B, 0x01, 0x01, 0x01, 0x03, 0x08, 0x04, 0x0E, 0x7F, 0x00, 0x01,
                0x0B, 0x01, 0x01, 0x01, 0x03, 0x08, 0x04, 0x0E, 0x7F, 0x00, 0x01, 0x0B, 0x01, 0x01,
                0x01, 0x03, 0x08, 0x04, 0x0E, 0x7F, 0x00, 0x01, 0x0B, 0x01, 0x01, 0x01, 0x04, 0x0E,
                0x7F, 0x00, 0x01, 0x0B, 0x01, 0x01, 0x01, 0x04, 0x0E, 0x7F, 0x00, 0x01, 0x0B, 0x01,
                0x01, 0x01, 0x02, 0x0A, 0x04, 0x0E, 0x7F, 0x00, 0x01, 0x0B, 0x01, 0x01, 0x01, 0x02,
                0x0A, 0x04, 0x0E, 0x7F, 0x00, 0x01, 0x0B, 0x01, 0x01, 0x01, 0x02, 0x0A, 0x04, 0x0E,
                0x7F, 0x00, 0x01, 0x0B, 0x01, 0x01, 0x01, 0x02, 0x0A, 0x04, 0x0E, 0x7F, 0x00, 0x01,
                0x0B, 0x01, 0x01, 0x01, 0x02, 0x0A, 0x04, 0x0E, 0x06, 0x01, 0x7F, 0x00, 0x01, 0x0B,
                0x01, 0x01, 0x01, 0x02, 0x0A, 0x04, 0x0E, 0x06, 0x01, 0x7F, 0x02, 0x0A, 0x7F, 0x02,
                0x0A, 0x7F, 0xF7, // End for report capabilities
                0xF0, 0x6A, 0x7F, 0x7F, 0x7F, 0xF7, // Result for report analog mapping
            ])),
            write_buf: Arc::new(RwLock::new(vec![0; 100])),
            read_index: Arc::new(Default::default()),
            write_index: Arc::new(Default::default()),
        }
    }
}

impl MockTransport {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            connected: Arc::new(Default::default()),
            read_buf: Arc::new(RwLock::new(data)),
            write_buf: Arc::new(RwLock::new(vec![0; 100])),
            read_index: Arc::new(Default::default()),
            write_index: Arc::new(Default::default()),
        }
    }
}

impl Display for MockTransport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockTransport")
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl IoTransport for MockTransport {
    fn open(&self) -> Result<(), Error> {
        pause_sync!(100);
        self.connected.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn close(&self) -> Result<(), Error> {
        pause_sync!(100);
        self.connected.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn write(&self, buf: &[u8]) -> Result<(), Error> {
        let mut write_buf = self.write_buf.write();
        let mut index = self.write_index.write();

        let len = write_buf.len().min(buf.len());
        if *index + len > write_buf.len() {
            *index = 0;
        }

        write_buf[*index..*index + len].copy_from_slice(&buf[..len]);
        *index += len;

        Ok(())
    }

    fn read_exact(&self, buf: &mut [u8]) -> Result<(), Error> {
        let read_buf = self.read_buf.read();
        let mut index = self.read_index.write();

        let len = read_buf.len().min(buf.len());
        if *index + len > read_buf.len() {
            *index = 0;
        }

        buf[..len].copy_from_slice(&read_buf[*index..*index + len]);
        *index += len;

        Ok(())
    }
}
