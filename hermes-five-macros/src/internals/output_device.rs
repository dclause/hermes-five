use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type};

/// Macro to automatically implement the OutputDevice trait for a struct.
/// It injects `state`, `default`, and `animation` fields if not present,
/// and generates default trait methods based on the provided state type.
pub fn output_device_macro(args: TokenStream, input: TokenStream) -> TokenStream {
    // Resolve reference to `hermes_five` crate
    let hermes_five = match crate_name("hermes_five") {
        Ok(FoundCrate::Itself) | Err(_) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!(::#ident)
        }
    };

    // Parse the input struct
    let input: DeriveInput = syn::parse2(input).expect("Failed to parse input struct");

    // Parse the type argument (e.g., u16)
    let state_type: Type = syn::parse2(args).expect("State type argument is required");

    let struct_name = &input.ident;
    let vis = &input.vis;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let attrs = &input.attrs;

    // Extract and validate struct fields
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input.ident,
                    "The #[output_device] macro only supports structs with named fields",
                )
                .to_compile_error();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                &input.ident,
                "The #[output_device] macro can only be used on structs",
            )
            .to_compile_error();
        }
    };

    let mut fields = fields.clone();

    // Check if 'state' field exists and validate its type
    let state_field = fields.iter().find(|field| {
        field
            .ident
            .as_ref()
            .map(|ident| ident == "state")
            .unwrap_or(false)
    });

    let output_impl = match state_field {
        Some(field) => {
            let state_field_type = &field.ty;
            let state_type_str = quote!(#state_field_type).to_string().replace(" ", "");

            if !state_type_str.starts_with("Arc<")
                && !state_type_str.starts_with("std::sync::Arc<")
                && !state_type_str.starts_with("sync::Arc<")
            {
                return syn::Error::new_spanned(
                    field,
                    format!(
                        "The 'state' field must be of type 'Arc<{}>' or a compatible type.\n\
                        Found: {}\n\
                        Expected: Arc<{}>",
                        quote!(#state_type),
                        quote!(#state_type_str),
                        quote!(#state_type)
                    ),
                )
                .to_compile_error();
            }

            quote!()
        }
        None => {
            fields.push(syn::parse_quote! {
                /// Shared, mutable device state.
                state: std::sync::Arc<parking_lot::RwLock<#state_type>>
            });

            quote! {
                impl #impl_generics #struct_name #ty_generics #where_clause {
                    #[inline(always)]
                    fn get_value(&self) -> #state_type {
                        *self.state.read()
                    }

                    #[inline(always)]
                    fn set_value(&self, value: #state_type) {
                        *self.state.write() = value;
                    }
                }
            }
        }
    };

    // Inject default and animation fields
    fields.push(syn::parse_quote! {
        /// The device's default state.
        default: #state_type
    });

    fields.push(syn::parse_quote! {
        /// Animation task handler.
        #[cfg_attr(feature = "serde", serde(skip))]
        animation: std::sync::Arc<Option<#hermes_five::animations::Animation>>
    });

    // Generate the struct with additional fields
    let original_struct = quote! {
        #(#attrs)*
        #vis struct #struct_name #generics {
            #fields
        }
    };

    // Basic Device impl (empty for now)
    let device_impl = quote! {
        #[cfg_attr(feature = "serde", typetag::serde)]
        impl #impl_generics #hermes_five::devices::Device for #struct_name #ty_generics #where_clause {}
    };

    // OutputDevice trait implementation
    let output_device_impl = quote! {
        #[cfg_attr(feature = "serde", typetag::serde)]
        impl #impl_generics #hermes_five::devices::OutputDevice for #struct_name #ty_generics #where_clause {
            #[inline(always)]
            fn get_state(&self) -> #hermes_five::utils::State {
                self.get_value().into()
            }

            fn set_state(&mut self, state: #hermes_five::utils::State) -> Result<#hermes_five::utils::State, #hermes_five::errors::Error> {
                let value = self.parse_state(state)?;
                if self.get_value() == value {
                    return Ok(value.into());
                }
                self.apply_value(value)?;
                self.set_value(value);
                Ok(value.into())
            }

            #[inline(always)]
            fn get_default(&self) -> #hermes_five::utils::State {
                self.default.into()
            }

            fn reset(&mut self) -> Result<#hermes_five::utils::State, #hermes_five::errors::Error> {
                self.stop();
                let value = self.parse_state(self.get_default())?;
                self.apply_value(value)?;
                self.set_value(value);
                Ok(value.into())
            }

            fn animate<S: Into<#hermes_five::utils::State>>(&mut self, state: S, duration: u64, transition: #hermes_five::animations::Easing) {
                self.stop();
                let mut animation = #hermes_five::animations::Animation::from(
                    #hermes_five::animations::Track::new(self.clone())
                        .with_keyframe(#hermes_five::animations::Keyframe::new(state, 0, duration).set_transition(transition)),
                );
                animation.play();
                self.animation = std::sync::Arc::new(Some(animation));
            }

            #[inline(always)]
            fn is_busy(&self) -> bool {
                self.animation.is_some()
            }

            fn stop(&mut self) {
                if let Some(animation) = std::sync::Arc::get_mut(&mut self.animation).and_then(Option::as_mut) {
                    animation.stop();
                }
                self.animation = std::sync::Arc::new(None);
            }
        }
    };

    // Combine all generated tokens
    let expanded = quote! {
        #original_struct
        #output_impl
        #device_impl
        #output_device_impl
    };

    expanded
}
