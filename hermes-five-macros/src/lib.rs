//! Defines Hermes-Five Runtime macro.

extern crate proc_macro;

use crate::internals::output_device::output_device_macro;
use crate::internals::runtime::{runtime_macro, TokioMode};
use proc_macro::TokenStream;

mod internals;

/// Macro definition for Hermes-Five Runtime.
///
/// This macro should probably be used once only in a project above your main.
/// It replaces the original tokio [`#[tokio::main]`] which it depends on.
///
/// _Executes the entire function in a blocking thread and provides synchronization for waiting on all
/// subsequently and dynamically created tasks (using `task::run`)._
#[proc_macro_attribute]
pub fn runtime(_: TokenStream, item: TokenStream) -> TokenStream {
    runtime_macro(item.into(), TokioMode::Main).into()
}

/// Defines `#[hermes_five_macros::test]` test macro.
#[proc_macro_attribute]
pub fn test(_: TokenStream, item: TokenStream) -> TokenStream {
    runtime_macro(item.into(), TokioMode::Test).into()
}

/// Creates the implementation of an `OutputDevice` for the annotation structure.
///
/// # Warning
///
/// 1. When provided with a `Type`, the state generated into a parking_lot RwLock and you *MUST*
/// provide the
/// ```no_run
///
/// // Use
/// #[output_device_macro(Type)]
/// struct CustomDevice;
///
/// // The generated structure will be:
/// struct CustomDevice {
///     state: std::sync::Arc<parking_lot::RwLock<Type>>,
///     default: Type,
///     animation: std::sync::Arc<Option<hermes_five::animations::Animation>>,
/// }
///
/// // You MUST provide at least the following methods:
/// impl CustomDevice {
///     /// Parses the generic `State` enum into the device's specific `Value`.
///     fn parse_state(&self, state: hermes_five::utils::State) -> Result<Type, hermes_five::errors::Error> {
///         todo!()
///     }
///
///     /// Applies the specific value to the hardware.
///     fn apply_value(&mut self, value: Type) -> Result<(), hermes_five::errors::Error> {
///         todo!()
///     }
/// }
/// ```
///
/// 2. You _can_ choose to store the state into a different type then the provided one.
///    In that case, you must provide some more functions:
/// ```no_run
/// // Use
/// use std::sync::atomic::AtomicU16;
/// use hermes_five_macros::output_device;
///
/// // Assuming any type here:
/// type CustomType = AtomicU16;
/// type Type = u16;
///
/// #[output_device(Type)]
/// struct CustomDevice {
///     state: std::sync::Arc<CustomType>
/// }
///
/// // The generated structure will be:
/// struct CustomDevice {
///     state: std::sync::Arc<CustomType>,
///     default: Type,
///     animation: std::sync::Arc<Option<hermes_five::animations::Animation>>,
/// }
///
/// // You MUST provide at least the following methods:
/// impl CustomDevice {
///
///     // Returns the value of `self.state` as `Type`.
///     fn get_value(&self) -> Type { todo!() }
///
///     // Sets the value of `self.state` from a given `Type`.
///     fn set_value(&self, value: Type) { todo!() }
///
///     // Parses the generic `State` enum into the device's specific `Value`.
///     fn parse_state(&self, state: hermes_five::utils::State) -> Result<Type, hermes_five::errors::Error> {
///         todo!()
///     }
///
///     // Applies the specific value to the hardware.
///     fn apply_value(&mut self, value: Type) -> Result<(), hermes_five::errors::Error> {
///         todo!()
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn output_device(attr: TokenStream, item: TokenStream) -> TokenStream {
    output_device_macro(attr.into(), item.into()).into()
}
