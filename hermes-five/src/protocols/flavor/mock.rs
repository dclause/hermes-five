use crate::protocols::{Error, Protocol, ProtocolHardware};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default)]
pub struct MockProtocol {
    pub hardware: ProtocolHardware,
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Protocol for MockProtocol {
    fn hardware(&self) -> &ProtocolHardware {
        &self.hardware
    }

    fn hardware_mut(&mut self) -> &mut ProtocolHardware {
        &mut self.hardware
    }

    fn open(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        // Simulate write operation (for testing purposes)
        Ok(())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        // Simulate read operation (for testing purposes)
        Ok(())
    }
}
