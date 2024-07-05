//! Defines Hermes-Five Runtime macro.

#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn::{Fields, ItemFn, ItemStruct, parse_macro_input};
use syn::parse::{Nothing, Parser};

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
pub fn runtime(_: TokenStream, item: TokenStream) -> TokenStream {
    macro_inner(item, false)
}

/// Same as `#[hermes_macros::runtime]` but for tests.
#[proc_macro_attribute]
pub fn test(_: TokenStream, item: TokenStream) -> TokenStream {
    macro_inner(item, true)
}

/// Same as `#[hermes_macros::runtime]` but for tests.
fn macro_inner(item: TokenStream, test: bool) -> TokenStream {
    let hermes_five = hermes_five_crate_path();

    let input = parse_macro_input!(item as ItemFn);
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;

    // Define the #[tokio::main] / #[tokio::test] tokio macro attribute.
    let tokio_main_attr = match test {
        #[cfg(test)]
        true => quote! {
            #[#hermes_five::utils::tokio::test]
            #[#hermes_five::utils::serial_test::serial]
        },
        _ => quote! {
            #[#hermes_five::utils::tokio::main]
        },
    };

    let modified_block = quote! {
        {
            // Channel for communicating task completions.
            // The arbitrary hardcoded limit is 50 concurrent running tasks.
            let (sender, mut receiver) = #hermes_five::utils::tokio::sync::mpsc::channel::<tokio::task::JoinHandle<()>>(100);

            // Update the global task sender
            {
                let mut write_guard = #hermes_five::utils::task::SENDER.write().unwrap();
                *write_guard = Some(sender.clone());
            }

            #block

            {
                let mut w = #hermes_five::utils::task::SENDER.write().unwrap();
                *w = None;
            }
            drop(sender); // Drop the cloned sender to close the channel

            // Wait for all dynamically spawned tasks to complete.
            while let Some(handle) = receiver.recv().await {
                handle
                    .await
                    .expect("Failed to join dynamically spawned task");
            }
        }
    };

    // Reconstruct the function with the modified block
    let output = quote! {
        #tokio_main_attr
        #(#attrs)*
        #vis #sig
        #modified_block
    };

    // Return the modified function as a TokenStream
    output.into()
}

// #################################################################################

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Entity(args: TokenStream, input: TokenStream) -> TokenStream {
    let crate_path = hermes_five_crate_path();
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let _ = parse_macro_input!(args as Nothing);

    if let Fields::Named(ref mut fields) = item_struct.fields {
        fields.named.push(
            syn::Field::parse_named
                .parse2(quote! { pub(crate) id: #crate_path::storage::entity::Id })
                .unwrap(),
        );
    }

    let name = &item_struct.ident;

    return quote! {
        use #crate_path::storage::typetag;

        #[derive(#crate_path::storage::serde::Serialize, #crate_path::storage::serde::Deserialize)]
        #item_struct

        #[typetag::serde]
        impl #crate_path::storage::entity::Entity for #name {
            fn get_id(&self) -> #crate_path::storage::entity::Id {
                self.id
            }
            fn set_id(&mut self, id: #crate_path::storage::entity::Id) {
                self.id = id
            }
        }
    }
    .into();
}

// #################################################################################

/// Determines what crate name should be used to refer to `hermes_core`.
/// crate::... or hermes_five::... depending.
fn hermes_five_crate_path() -> syn::Path {
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
