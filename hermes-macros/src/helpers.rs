extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;

/// Determines what crate name should be used to refer to `hermes_core`.
/// crate::... or hermes_five::... depending.
pub fn hermes_five_crate_path() -> TokenStream {
    let is_internal = std::env::var("CARGO_CRATE_NAME")
        .map(|pkg_name| pkg_name == "hermes_five")
        .unwrap_or_default();

    #[cfg(doctest)]
    let is_internal = false;

    match is_internal {
        true => quote!(crate),
        false => quote!(hermes_five),
    }
}
