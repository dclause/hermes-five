use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{parse2, ItemEnum, Token};

/// Struct representing the macro input:
/// e.g.
///     Board,
///     pub enum BoardEvent { ... }
struct EventMacroInput {
    class_ident: Ident,
    _comma: Token![,],
    event_enum: ItemEnum,
}

impl syn::parse::Parse for EventMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let class_ident: Ident = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let event_enum: ItemEnum = input.parse()?;

        Ok(EventMacroInput {
            class_ident,
            _comma,
            event_enum,
        })
    }
}

/// Generates events from a given event enum.
///
/// The generated methods are:
/// - `.on(event: EnumEvent, handler) ....`
/// as well as helper methods for each event:
/// - `.on_event1(handler) ....`
/// - `.on_event2(handler) ....`
///
/// Example:
///
/// ```exclude
/// use hermes_five_macros::generate_events;
///
/// generate_events! {
///     Board,
///     /// Lists all events a Board can emit/listen.
///     #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
///     pub enum BoardEvent {
///         /// Triggered when the board connection is established.
///         OnReady,
///         /// Triggered when the board connection is closed.
///         OnClosed,
///     }
/// }
/// ```
pub fn generate_events(input: TokenStream) -> TokenStream {
    let input: EventMacroInput = match parse2(input) {
        Ok(data) => data,
        Err(err) => return err.to_compile_error(),
    };

    let class_name = input.class_ident;
    let input_enum = input.event_enum;

    let enum_vis = &input_enum.vis;
    let enum_ident = &input_enum.ident;
    let enum_attrs = &input_enum.attrs;
    let enum_variants = &input_enum.variants;

    // Generate the enum as-is
    let enum_definition = quote! {
        #(#enum_attrs)*
        #enum_vis enum #enum_ident {
            #enum_variants
        }
    };

    let doc_str = format!(
        "Registers a callback to be executed on a given event.\n\n\
     Available events are defined by the enum: [`{}`].",
        enum_ident
    );

    // Generate the generic .on(...) method
    let on_method = quote! {
        #[doc = #doc_str]
        pub fn on<F, Fut, R>(&self, event: #enum_ident, handler: F)
        where
            F: Fn(#class_name) -> Fut + Send + Sync + 'static,
            Fut: std::future::Future<Output = R> + Send + 'static,
            R: Into<GenericResult>,
        {
            self.events.on(event, handler);
        }
    };

    // Generate on_<variant>() methods
    let mut variant_methods = Vec::new();

    for variant in enum_variants {
        let variant_ident = &variant.ident;
        let variant_attrs = &variant.attrs;

        // Build the alias method name: strip 'On' prefix if any from the event name.
        let variant_name_str = variant_ident.to_string();
        let stripped = match variant_name_str.strip_prefix("On") {
            Some(stripped) => stripped,
            None => &variant_name_str,
        };
        let method_name_str = format!("on_{}", stripped.to_lowercase());
        let method_ident = format_ident!("{}", method_name_str);

        let doc_str = format!(
            "<br/>_Alias method for `.on({}::{}, ...)` - see [Self::on]._",
            enum_ident, variant_ident
        );

        let method = quote! {
            #(#variant_attrs)*
            #[doc = #doc_str]
            pub fn #method_ident <F, Fut, R>(&self, handler: F)
            where
                F: Fn(#class_name) -> Fut + Send + Sync + 'static,
                Fut: std::future::Future<Output = R> + Send + 'static,
                R: Into<GenericResult>,
            {
                self.on(#enum_ident::#variant_ident, handler);
            }
        };

        variant_methods.push(method);
    }

    let expanded = quote! {
        #enum_definition
        impl #class_name {
            #on_method
            #(#variant_methods)*
        }
    };

    expanded.into()
}
