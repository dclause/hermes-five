use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Fields, ItemStruct, parse_macro_input};
use syn::parse::{Nothing, Parser};

use crate::helpers::hermes_five_crate_path;

/// Internal redefinition of `#[Entity]`.
///
/// This method uses proc_macro2 to allow runtime macro expansion. This is done only for testing coverage
/// purpose through the runtime-macros crate.
/// @see Tarpaulin README https://docs.rs/crate/cargo-tarpaulin/latest
/// @see runtime-macros https://docs.rs/runtime-macros/latest/runtime_macros
pub fn entity_macro_internal(args: TokenStream2, input: TokenStream2) -> TokenStream2 {
    entity_macro(args.into(), input.into()).into()
}

/// See `#[Entity]` for details.
fn entity_macro(args: TokenStream, input: TokenStream) -> TokenStream {
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

#[cfg(test)]
mod tests {
    use runtime_macros::emulate_attributelike_macro_expansion;

    use crate::entity_macro::entity_macro_internal;

    #[test]
    fn code_coverage() {
        // This code doesn't check much. Instead, it does macro expansion at run time to let
        // tarpaulin measure code coverage for the macro.
        let file = std::fs::File::open("tests/entity_macro.rs").unwrap();
        emulate_attributelike_macro_expansion(file, &[("runtime", entity_macro_internal)]).unwrap();
    }

    #[test]
    fn syntax_error() {
        // This code makes sure that the given file doesn't compile.
        let file = std::fs::File::open("tests/entity_macro.rs").unwrap();
        emulate_attributelike_macro_expansion(file, &[("runtime", entity_macro_internal)]).unwrap();
    }
}
