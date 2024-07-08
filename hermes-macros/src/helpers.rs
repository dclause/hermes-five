/// Determines what crate name should be used to refer to `hermes_core`.
/// crate::... or hermes_five::... depending.
pub fn hermes_five_crate_path() -> syn::Path {
    let is_internal = std::env::var("CARGO_CRATE_NAME")
        .map(|pkg_name| pkg_name == "hermes_five")
        .unwrap_or_default();

    #[cfg(doctest)]
    let is_internal = false;

    if is_internal {
        syn::parse_quote!(crate)
    } else {
        syn::parse_quote!(hermes_five)
    }
}
