//! Defines Hermes-Five Runtime macro.

#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

extern crate proc_macro;

use proc_macro::TokenStream;

use crate::entity_macro::entity_macro_internal;
use crate::runtime_macro::{runtime_macro_internal, test_macro_internal};

mod entity_macro;
mod helpers;
mod runtime_macro;

/// Macro definition for Hermes-Five Runtime.
///
/// This macro should be used once only in a project.
/// This macro requires `tokio` as a dependency.
///
/// _Executes the entire function in a blocking thread and provides synchronization for waiting on all
/// subsequently and dynamically created threads (using `task::run`)._
///
/// # Example
/// ```
/// #[hermes_macros::runtime]
/// async fn main() {
///     // whatever
/// }
/// ```
#[proc_macro_attribute]
pub fn runtime(args: TokenStream, item: TokenStream) -> TokenStream {
    runtime_macro_internal(args.into(), item.into()).into()
}

/// Defines `#[hermes_macros::runtime]` test macro.
#[proc_macro_attribute]
pub fn test(args: TokenStream, item: TokenStream) -> TokenStream {
    test_macro_internal(args.into(), item.into()).into()
}

// #################################################################################

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Entity(args: TokenStream, input: TokenStream) -> TokenStream {
    entity_macro_internal(args.into(), input.into()).into()
}

// #################################################################################
