use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ItemFn, parse_macro_input, ReturnType, Stmt};

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
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(item as ItemFn);

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
        ReturnType::Type(_, _) => true,
    };

    // Extract the last statement if it's an expression (potential return value)
    let return_expr = match stmts.last() {
        Some(Stmt::Expr(_expr, ..)) => {
            if let Some(last) = stmts.pop() {
                Some(last)
            } else {
                None
            }
        }
        _ => None,
    };

    // Define the #[tokio::main] / #[tokio::test] tokio macro attribute.
    let tokio_main_attr = match test {
        true => quote! {#[#hermes_five::utils::tokio::test]},
        _ => quote! {#[#hermes_five::utils::tokio::main]},
    };

    // Generate the function body
    let mut body = vec![quote! {
        // Insert custom code before the original function body
        #hermes_five::utils::task::init_task_channel().await;
    }];

    // Insert the original function body statements
    body.extend(stmts.into_iter().map(|stmt| quote! { #stmt }));

    // Insert custom code after the original function body
    body.push(quote! {
        let cell = #hermes_five::utils::task::RUNTIME_RX.get().ok_or(#hermes_five::protocols::RuntimeError).unwrap();
        let mut lock = cell.lock().await;
        let receiver = lock.as_mut().ok_or(#hermes_five::protocols::RuntimeError).unwrap();

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
    } else if !has_return_type {
        // Add an empty tuple return if needed
        body.push(quote! { () });
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
    use runtime_macros::emulate_attributelike_macro_expansion;

    use crate::runtime_macro::{runtime_macro_internal, test_macro_internal};

    #[test]
    fn code_coverage() {
        // This code doesn't check much. Instead, it does macro expansion at run time to let
        // tarpaulin measure code coverage for the macro.
        let file = std::fs::File::open("tests/runtime_macro.rs").unwrap();
        emulate_attributelike_macro_expansion(file, &[("runtime", runtime_macro_internal)])
            .unwrap();

        let file = std::fs::File::open("tests/runtime_macro.rs").unwrap();
        emulate_attributelike_macro_expansion(file, &[("test", test_macro_internal)]).unwrap();
    }

    #[test]
    fn syntax_error() {
        // This code makes sure that the given file doesn't compile.
        let file = std::fs::File::open("tests/compile_fail/incorrect_runtime.rs").unwrap();
        emulate_attributelike_macro_expansion(file, &[("runtime", runtime_macro_internal)])
            .unwrap();
        let file = std::fs::File::open("tests/compile_fail/incorrect_runtime.rs").unwrap();
        emulate_attributelike_macro_expansion(file, &[("test", test_macro_internal)]).unwrap();
    }
}
