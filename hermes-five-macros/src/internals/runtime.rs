use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemFn, Signature};

pub enum TokioMode {
    Main,
    Test,
}

/// See `#[hermes_five_macros::runtime]` for details in [`hermes-five-macros`] crate.
///
/// This method uses proc_macro2 TokenStream in order to allow easier testing and code coverage checks.
/// This is the only benefice to have it as a sub-method here (and have it bound to sub-crate hermes-five-macros-internals)
pub fn runtime_macro(item: TokenStream, tokio: TokioMode) -> TokenStream {
    let hermes_five = quote!(::hermes_five);
    // Parse the input tokens into a syntax tree
    let input: ItemFn = syn::parse2(item).unwrap();

    // Destructure the input ItemFn
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;

    // Create a new Signature without `async`
    let sync_sig = Signature {
        asyncness: None,
        ..sig
    };

    // Extract the block's statements
    let stmts = block.stmts;

    // Define the #[test] attribute.
    let test_attr = match tokio {
        TokioMode::Main => quote! {},
        TokioMode::Test => quote! {#[test]},
    };

    let is_test = matches!(tokio, TokioMode::Test);

    // Generate the expanded function
    quote! {
        #test_attr
        #(#attrs)*
        #vis #sync_sig {
            let rt = #hermes_five::utils::task::setup_rt(#is_test);
            rt.block_on(async {
                #(#stmts)*
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use crate::internals::{runtime_macro, TokioMode};

    #[test]
    fn test_runtime_macro_result() {
        let input = quote! {
            async fn main() -> Result<(), Error> {
                let x = 3;
                Ok(())
            }
        };

        let control = quote! {
            fn main() -> Result<(), Error> {
                let rt = ::hermes_five::utils::task::setup_rt(false);
                rt.block_on(async {
                    let x = 3;
                    Ok(())
                })
            }
        };

        let output = runtime_macro(input, TokioMode::Main);
        assert_eq!(
            format!("{}", output),
            format!("{}", control),
            "Macro expansion for runtime should be correct with Ok(())."
        );
    }

    #[test]
    fn test_runtime_macro_no_result() {
        let input = quote! {
            async fn main() {
                let x = 3;
                blabla.await;
            }
        };

        let control = quote! {
            fn main() {
                let rt = ::hermes_five::utils::task::setup_rt(false);
                rt.block_on(async {
                    let x = 3;
                    blabla.await;
                })
            }
        };

        let output = runtime_macro(input, TokioMode::Main);
        assert_eq!(
            format!("{}", output),
            format!("{}", control),
            "Macro expansion for runtime should be correct with no result."
        );
    }

    #[test]
    fn test_runtime_macro_explicit_void() {
        let input = quote! {
            async fn main() -> () {
                let x = 3;
                ()
            }
        };

        let control = quote! {
            fn main() -> () {
                let rt = ::hermes_five::utils::task::setup_rt(false);
                rt.block_on(async {
                    let x = 3;
                    ()
                })
            }
        };

        let output = runtime_macro(input, TokioMode::Main);
        assert_eq!(
            format!("{}", output),
            format!("{}", control),
            "Macro expansion for runtime should be correct with explicit void."
        );
    }

    #[test]
    fn test_runtime_macro_test() {
        let input = quote! {
            async fn main() { }
        };

        let control = quote! {
            #[test]
            fn main() {
                let rt = ::hermes_five::utils::task::setup_rt(true);
                rt.block_on(async { })
            }
        };

        let output = runtime_macro(input, TokioMode::Test);
        assert_eq!(
            format!("{}", output),
            format!("{}", control),
            "Macro expansion for test mode should be correct."
        );
    }
}
