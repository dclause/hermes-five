use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{parse2, DeriveInput};

pub fn derive_expander(input: TokenStream) -> TokenStream {
    let hermes_five = match crate_name("hermes_five") {
        Ok(FoundCrate::Itself) | Err(_) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!(::#ident)
        }
    };

    let ast: DeriveInput = match parse2(input) {
        Ok(ast) => ast,
        Err(err) => return err.to_compile_error(),
    };
    let name = &ast.ident;
    let expanded = quote! {
        impl #hermes_five::hardware::Expander for #name {}
    };
    expanded
}
