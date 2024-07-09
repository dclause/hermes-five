use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

use crate::helpers::hermes_five_crate_path;

/// Internal redefinition of `#[hermes_macro::runtime]`.
///
/// This method uses proc_macro2 to allow runtime macro expansion. This is done only for testing coverage
/// purpose through the runtime-macros crate.
/// @see Tarpaulin README https://docs.rs/crate/cargo-tarpaulin/latest
/// @see runtime-macros https://docs.rs/runtime-macros/latest/runtime_macros
pub fn runtime_macro_internal(_: TokenStream2, item: TokenStream2) -> TokenStream2 {
    runtime_macro(item.into(), false).into()
}

/// Internal redefinition of `#[hermes_macro::test]`.
///
/// This method uses proc_macro2 to allow runtime macro expansion. This is done only for testing coverage
/// purpose through the runtime-macros crate.
/// @see Tarpaulin README https://docs.rs/crate/cargo-tarpaulin/latest
/// @see runtime-macros https://docs.rs/runtime-macros/latest/runtime_macros
pub fn test_macro_internal(_: TokenStream2, item: TokenStream2) -> TokenStream2 {
    runtime_macro(item.into(), true).into()
}

/// See `#[hermes_macros::runtime]` for details.
pub fn runtime_macro(item: TokenStream, test: bool) -> TokenStream {
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
        true => quote! {
            #[#hermes_five::utils::tokio::test]
        },
        _ => quote! {
            #[#hermes_five::utils::tokio::main]
        },
    };

    let modified_block = quote! {
        {
            let mut lock = #hermes_five::utils::task::RECEIVER
                .get_or_init(|| async {
                    // If we need to init a receiver, that also mean we need to init a sender.
                    let (sender, mut receiver) =
                        #hermes_five::utils::tokio::sync::mpsc::channel::<tokio::task::JoinHandle<()>>(100);

                    if (#hermes_five::utils::task::SENDER.initialized()) {
                        panic!("A sender exists while a receiver don't");
                    }

                    let _ =  #hermes_five::utils::task::SENDER
                        .get_or_init(|| async {
                            #hermes_five::utils::tokio::sync::RwLock::new(Some(sender.clone()))
                        })
                        .await;
                    #hermes_five::utils::tokio::sync::RwLock::new(Some(receiver))
                })
                .await
                .write()
                .await;
            let receiver = lock.as_mut().unwrap();

            #block

            // Wait for all dynamically spawned tasks to complete.
            while receiver.len() > 0 {
                if let Some(handle) = receiver.recv().await {
                    handle
                        .await
                        .expect("Failed to join dynamically spawned task");
                }
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

#[cfg(test)]
mod tests {
    use runtime_macros::emulate_attributelike_macro_expansion;

    use crate::runtime_macro::{runtime_macro_internal, test_macro_internal};

    #[test]
    fn code_coverage() {
        // This code doesn't check much. Instead, it does macro expansion at run time to let
        // tarpaulin measure code coverage for the macro.
        let mut path = std::env::current_dir().unwrap();
        path.push("tests");
        path.push("runtime_macro.rs");
        let file = std::fs::File::open(path.clone()).unwrap();
        emulate_attributelike_macro_expansion(file, &[("runtime", runtime_macro_internal)])
            .unwrap();
        let file = std::fs::File::open(path).unwrap();
        emulate_attributelike_macro_expansion(file, &[("test", test_macro_internal)]).unwrap();
    }

    #[test]
    fn syntax_error() {
        // This code makes sure that the given file doesn't compile.
        let mut path = std::env::current_dir().unwrap();
        path.push("tests");
        path.push("compile_fail");
        path.push("incorrect_runtime.rs");
        let file = std::fs::File::open(path.clone()).unwrap();
        emulate_attributelike_macro_expansion(file, &[("runtime", runtime_macro_internal)])
            .unwrap();
        let file = std::fs::File::open(path).unwrap();
        emulate_attributelike_macro_expansion(file, &[("test", test_macro_internal)]).unwrap();
    }
}
