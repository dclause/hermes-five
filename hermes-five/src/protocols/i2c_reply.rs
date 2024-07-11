/// An I2C reply.
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct I2CReply {
    pub address: i32,
    pub register: i32,
    pub data: Vec<u8>,
}
