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

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test_hermes_five_crate_path_internal() {
        // Set the environment variable
        env::set_var("CARGO_CRATE_NAME", "hermes_five");

        // Call the function
        let result = hermes_five_crate_path().to_string();

        // Assert the result
        assert_eq!(result, "crate");

        // Clean up the environment variable
        env::remove_var("CARGO_CRATE_NAME");
    }

    #[test]
    fn test_hermes_five_crate_path_external() {
        // Set the environment variable to something else
        env::set_var("CARGO_CRATE_NAME", "some_other_crate");

        // Call the function
        let result = hermes_five_crate_path().to_string();

        // Assert the result
        assert_eq!(result, "hermes_five");

        // Clean up the environment variable
        env::remove_var("CARGO_CRATE_NAME");
    }

    #[test]
    fn test_hermes_five_crate_path_no_env_var() -> () {
        // Ensure the environment variable is not set
        env::remove_var("CARGO_CRATE_NAME");

        // Call the function
        let result = hermes_five_crate_path().to_string();

        // Assert the result
        assert_eq!(result, "hermes_five");
    }
}