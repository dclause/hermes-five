use std::io::{Read, Write};
use std::time::Duration;

use serialport::{
    ClearBuffer, DataBits, Error, ErrorKind, FlowControl, Parity, SerialPort, StopBits,
};

#[derive(Debug, Default, Clone)]
pub struct SerialPortMock {
    error: Option<Error>,
}

impl SerialPortMock {
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            error: Some(Error::new(kind, "Mock error reason")),
        }
    }
}
impl SerialPort for SerialPortMock {
    fn name(&self) -> Option<String> {
        Some(String::from("SerialPortMock"))
    }

    fn baud_rate(&self) -> serialport::Result<u32> {
        match self.error {
            None => Ok(37500),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn data_bits(&self) -> serialport::Result<DataBits> {
        match self.error {
            None => Ok(DataBits::Eight),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn flow_control(&self) -> serialport::Result<FlowControl> {
        match self.error {
            None => Ok(FlowControl::None),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn parity(&self) -> serialport::Result<Parity> {
        match self.error {
            None => Ok(Parity::None),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn stop_bits(&self) -> serialport::Result<StopBits> {
        match self.error {
            None => Ok(StopBits::One),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(1)
    }

    fn set_baud_rate(&mut self, _: u32) -> serialport::Result<()> {
        match self.error {
            None => Ok(()),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn set_data_bits(&mut self, _: DataBits) -> serialport::Result<()> {
        match self.error {
            None => Ok(()),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn set_flow_control(&mut self, _: FlowControl) -> serialport::Result<()> {
        match self.error {
            None => Ok(()),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn set_parity(&mut self, _: Parity) -> serialport::Result<()> {
        match self.error {
            None => Ok(()),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn set_stop_bits(&mut self, _: StopBits) -> serialport::Result<()> {
        match self.error {
            None => Ok(()),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn set_timeout(&mut self, _: Duration) -> serialport::Result<()> {
        match self.error {
            None => Ok(()),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn write_request_to_send(&mut self, _: bool) -> serialport::Result<()> {
        match self.error {
            None => Ok(()),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn write_data_terminal_ready(&mut self, _: bool) -> serialport::Result<()> {
        match self.error {
            None => Ok(()),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn read_clear_to_send(&mut self) -> serialport::Result<bool> {
        match self.error {
            None => Ok(true),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn read_data_set_ready(&mut self) -> serialport::Result<bool> {
        match self.error {
            None => Ok(true),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn read_ring_indicator(&mut self) -> serialport::Result<bool> {
        match self.error {
            None => Ok(true),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn read_carrier_detect(&mut self) -> serialport::Result<bool> {
        match self.error {
            None => Ok(true),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn bytes_to_read(&self) -> serialport::Result<u32> {
        match self.error {
            None => Ok(3),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn bytes_to_write(&self) -> serialport::Result<u32> {
        match self.error {
            None => Ok(3),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn clear(&self, _: ClearBuffer) -> serialport::Result<()> {
        match self.error {
            None => Ok(()),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn try_clone(&self) -> serialport::Result<Box<dyn SerialPort>> {
        match self.error {
            None => Ok(Box::new(self.clone())),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn set_break(&self) -> serialport::Result<()> {
        match self.error {
            None => Ok(()),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }

    fn clear_break(&self) -> serialport::Result<()> {
        match self.error {
            None => Ok(()),
            Some(_) => Err(self.error.clone().unwrap()),
        }
    }
}
impl Read for SerialPortMock {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.error {
            None => match buf.len() {
                0 => Ok(100),
                len => Ok(len),
            },
            Some(_) => Err(std::io::Error::from(std::io::ErrorKind::InvalidData)),
        }
    }
}

impl Write for SerialPortMock {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.error {
            None => match buf.len() {
                0 => Ok(100),
                len => Ok(len),
            },
            Some(_) => Err(std::io::Error::from(std::io::ErrorKind::InvalidData)),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self.error {
            None => Ok(()),
            Some(_) => Err(std::io::Error::from(std::io::ErrorKind::InvalidData)),
        }
    }
}
