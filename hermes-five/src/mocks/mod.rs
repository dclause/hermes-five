#![cfg_attr(coverage_nightly, coverage(off))]

//! Defines mock structure to test the implementations (requires `mock` feature flag).

mod mock_hardware;
mod mock_input_device;
mod mock_output_device;
mod mock_protocol;
mod mock_transport;

// re-export
pub use mock_hardware::*;
pub use mock_input_device::*;
pub use mock_output_device::*;
pub use mock_protocol::*;
pub use mock_transport::*;
