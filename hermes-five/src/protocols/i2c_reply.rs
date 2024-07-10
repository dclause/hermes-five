/// An I2C reply.
#[derive(Debug, Default, Clone)]
pub struct I2CReply {
    pub address: i32,
    pub register: i32,
    pub data: Vec<u8>,
}
