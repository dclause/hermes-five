//! Defines Hermes-Five Runtime macro.

extern crate proc_macro;

use proc_macro::TokenStream;

use crate::internals::{runtime_macro, TokioMode};

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

/// Defines `#[hermes_five_macros::runtime]` test macro.
#[proc_macro_attribute]
pub fn test(_: TokenStream, item: TokenStream) -> TokenStream {
    runtime_macro(item.into(), TokioMode::Test).into()
}
