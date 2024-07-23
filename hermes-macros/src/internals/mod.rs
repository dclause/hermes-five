use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemFn, ReturnType, Stmt};

use crate::internals::helpers::hermes_five_crate_path;

mod helpers;

pub enum TokioMode {
    Main,
    Test,
}

/// See `#[hermes_macros::runtime]` for details in [`hermes-macros`] crate.
///
/// This method uses proc_macro2 TokenStream in order to allow easier testing and tarpaulin code coverage.
/// This is the only benefice to have it as a sub-method here (and have it bound to sub-crate hermes-macros-internals)
pub fn runtime_macro(item: TokenStream, tokio: TokioMode) -> TokenStream {
    let hermes_five = hermes_five_crate_path();
    // Parse the input tokens into a syntax tree
    let input: ItemFn = syn::parse2(item).unwrap();

    // Destructure the input ItemFn
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;

    // Extract the block's statements
    let mut stmts = block.stmts;

    // Check if the function has an explicit return type
    let has_return_type = match &sig.output {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => match &**ty {
            syn::Type::Tuple(tuple) if tuple.elems.is_empty() => false,
            _ => true,
        },
    };

    // Extract the last statement if it's an expression (potential return value)
    let return_expr = if has_return_type {
        match stmts.pop() {
            Some(Stmt::Expr(expr, ..)) => Some(expr),
            Some(stmt) => {
                stmts.push(stmt);
                None
            }
            None => None,
        }
    } else {
        None
    };

    // Define the #[tokio::main] / #[tokio::test] tokio macro attribute.
    let tokio_main_attr = match tokio {
        TokioMode::Main => quote! {#[#hermes_five::utils::tokio::main]},
        TokioMode::Test => quote! {#[#hermes_five::utils::tokio::test]},
    };

    // Generate the function body
    let mut body = vec![quote! {
        #hermes_five::utils::task::init_task_channel().await;

        // // Original code
    }];

    // Insert the original function body statements
    // Check all "line-by-line" content within the body
    body.extend(stmts.into_iter().map(|stmt| match stmt {
        // In the case of an expression, we want to remove a null return "()" from the body
        // since it will be added later as a return_expr.
        Stmt::Expr(ref exp, _) => match exp {
            syn::Expr::Tuple(tuple) if tuple.elems.is_empty() => quote!(),
            _ => quote! { #stmt },
        },
        _ => quote! { #stmt },
    }));

    // Insert custom code after the original function body
    body.push(quote! {
        // ---

        let cell = #hermes_five::utils::task::RUNTIME_RX.get().ok_or(#hermes_five::errors::RuntimeError).unwrap();
        let mut lock = cell.lock();
        let receiver = lock.as_mut().ok_or(#hermes_five::errors::RuntimeError).unwrap();

        // Wait for all dynamically spawned tasks to complete.
        while receiver.len() > 0 {
            // We receive the task specific receiver.
            if let Some(mut task_receiver) = receiver.recv().await {

                // We receive the task result through that new receiver.
                if let Some(task_result) = task_receiver.recv().await {
                    match task_result {
                        #hermes_five::utils::task::TaskResult::Ok => {},
                        #hermes_five::utils::task::TaskResult::Err(err) => eprintln!("Task failed: {:?}", err.to_string()),
                    }
                }
            }
        }
    });

    // Add the return expression if there is one
    if let Some(return_stmt) = return_expr {
        body.push(quote! { #return_stmt });
    }

    // Generate the expanded function
    let expanded = quote! {
        #tokio_main_attr
        #(#attrs)*
        #vis #sig {
            #(#body)*
        }
    };

    // Return the generated TokenStream
    TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;

    use crate::internals::{runtime_macro, TokioMode};

    fn before() -> TokenStream {
        quote! {hermes_five::utils::task::init_task_channel().await;}
    }
    fn after() -> TokenStream {
        quote! {
            let cell = hermes_five::utils::task::RUNTIME_RX.get().ok_or(hermes_five::errors::RuntimeError).unwrap();
            let mut lock = cell.lock();
            let receiver = lock.as_mut().ok_or(hermes_five::errors::RuntimeError).unwrap();

            // Wait for all dynamically spawned tasks to complete.
            while receiver.len() > 0 {
                // We receive the task specific receiver.
                if let Some(mut task_receiver) = receiver.recv().await {

                    // We receive the task result through that new receiver.
                    if let Some(task_result) = task_receiver.recv().await {
                        match task_result {
                            hermes_five::utils::task::TaskResult::Ok => {},
                            hermes_five::utils::task::TaskResult::Err(err) => eprintln!("Task failed: {:?}", err.to_string()),
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn test_runtime_macro_result() {
        let before = before();
        let after = after();

        let input = quote! {
            async fn main() -> Result<(), Error> {
                let x = 3;
                Ok(())
            }
        };

        let control = quote! {
            #[hermes_five::utils::tokio::main]
            async fn main() -> Result<(), Error> {
                #before

                // Original code
                let x = 3;
                // ---

                #after

                Ok(())
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
        let before = before();
        let after = after();

        let input = quote! {
            async fn main() {
                let x = 3;
                blabla.await;
            }
        };

        let control = quote! {
            #[hermes_five::utils::tokio::main]
            async fn main() {
                #before

                // Original code
                let x = 3;
                blabla.await;
                // ---

                #after
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
        let before = before();
        let after = after();

        let input = quote! {
            async fn main() -> () {
                let x = 3;
                ()
            }
        };

        let control = quote! {
            #[hermes_five::utils::tokio::main]
            async fn main() -> () {
                #before

                // Original code
                let x = 3;
                // ---

                #after
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
        let before = before();
        let after = after();

        let input = quote! {
            async fn main() { }
        };

        let control = quote! {
            #[hermes_five::utils::tokio::test]
            async fn main() {
                #before

                // Original code
                // ---

                #after
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
