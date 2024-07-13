use crate::protocols::{Error, Protocol, ProtocolHardware};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default)]
pub struct MockProtocol {
    /// The base-protocol attributes.
    #[cfg_attr(feature = "serde", serde(skip))]
    hardware: ProtocolHardware,
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Protocol for MockProtocol {
    fn hardware(&self) -> &ProtocolHardware {
        &self.hardware
    }

    fn open(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn write(&mut self, _buf: &[u8]) -> Result<(), Error> {
        // Simulate write operation (for testing purposes)
        Ok(())
    }

    fn read_exact(&mut self, _buf: &mut [u8]) -> Result<(), Error> {
        // Simulate read operation (for testing purposes)
        Ok(())
    }
}
