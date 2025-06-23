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

/// Implements the `OutputDevice` trait for the annotated structure.
///
/// This macro injects the necessary fields and default trait implementations based on the specified state type.
/// You can use it in two modes, depending on whether you define the `state` field yourself.
///
/// # Usage
///
/// ## 1. Automatic State Injection
///
/// If the struct does **not** define a `state` field, the macro will inject:
///
/// ```ignore
/// type Type = u16;
///
/// #[output_device(Type)]
/// struct CustomDevice {}
///
/// // Expanded to:
/// // struct CustomDevice {
/// //     state: std::sync::Arc<parking_lot::RwLock<Type>>,
/// //     default: Type,
/// //     animation: std::sync::Arc<Option<hermes_five::animations::Animation>>,
/// // }
///
/// impl CustomDevice {
///     fn parse_state(&self, state: hermes_five::utils::State) -> Result<Type, hermes_five::errors::Error> {
///         todo!()
///     }
///
///     fn apply_value(&mut self, value: Type) -> Result<(), hermes_five::errors::Error> {
///         todo!()
///     }
/// }
/// ```
///
/// In this case, the `get_value` and `set_value` methods are generated automatically using the RwLock.
///
/// ## 2. Custom State Field
///
/// If the struct defines its own `state` field (e.g., using `AtomicU16`, `Mutex`, etc.),
/// you must provide additional methods to allow the macro to interact with it:
///
/// ```ignore
/// type Type = u16;
/// type CustomType = AtomicU16;
///
/// #[output_device(Type)]
/// struct CustomDevice {
///     state: std::sync::Arc<CustomType>,
/// }
///
/// // Expanded to:
/// // struct CustomDevice {
/// //     state: std::sync::Arc<CustomType>,
/// //     default: Type,
/// //     animation: std::sync::Arc<Option<hermes_five::animations::Animation>>,
/// // }
///
/// impl CustomDevice {
///     fn get_value(&self) -> Type {
///         todo!()
///     }
///
///     fn set_value(&self, value: Type) {
///         todo!()
///     }
///
///     fn parse_state(&self, state: hermes_five::utils::State) -> Result<Type, hermes_five::errors::Error> {
///         todo!()
///     }
///
///     fn apply_value(&mut self, value: Type) -> Result<(), hermes_five::errors::Error> {
///         todo!()
///     }
/// }
/// ```
///
/// # Notes
///
/// - The `Type` parameter must match the final value type used internally by the device (e.g. `u8`, `bool`, etc.).
/// - The `animation` field is included but can be a no-op if unused.
/// - This macro integrates with `hermes_five::devices::OutputDevice` and `Device`.
#[proc_macro_attribute]
pub fn output_device(attr: TokenStream, item: TokenStream) -> TokenStream {
    output_device_macro(attr.into(), item.into()).into()
}
