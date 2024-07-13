/// An I2C reply.
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct I2CReply {
    pub address: u16,
    pub register: u16,
    pub data: Vec<u16>,
}
