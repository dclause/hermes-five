#[cfg(test)]
pub use mock::MockProtocol;
pub use serial::SerialProtocol;

// mod io;
#[cfg(test)]
mod mock;
mod serial;
