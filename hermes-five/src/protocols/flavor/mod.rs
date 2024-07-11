#[cfg(test)]
pub use mock::MockProtocol;
pub use serial::SerialProtocol;

#[cfg(test)]
mod mock;
mod serial;
