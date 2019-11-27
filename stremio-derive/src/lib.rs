extern crate proc_macro;
use crate::proc_macro::TokenStream;

use proc_macro2::Span;
use proc_macro_crate::crate_name;
use quote::{quote, quote_spanned};
use std::borrow::Cow;
use std::env;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, FieldsNamed, Ident};

const CORE_CRATE_ORIGINAL_NAME: &str = "stremio-core";

#[proc_macro_derive(Model)]
pub fn model_derive(input: TokenStream) -> TokenStream {
    let core = core_ident().unwrap();
    let input = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
    }) = input.data
    {
        let name = &input.ident;
        let mut fields = named.iter();
        let first = fields.next().expect("at least one field required");
        assert!(
            first.ident.as_ref().map_or(false, |n| n == "ctx"),
            "first field must be named ctx"
        );
        // Using the explicit trait syntax, for more clear err messages
        let container_updates = fields.map(|f| {
            let name = &f.ident;
            quote_spanned! {f.span() =>
                .join(#core::state_types::UpdateWithCtx::update(&mut self.#name, &self.ctx, msg))
            }
        });
        let expanded = quote! {
            impl #core::state_types::Update for #name {
                fn update(&mut self, msg: &#core::state_types::messages::Msg) -> #core::state_types::Effects {
                    #core::state_types::Update::update(&mut self.ctx, msg)
                        #(#container_updates)*
                }
            }
        };

        TokenStream::from(expanded)
    } else {
        panic!("#[derive(Model)] is only defined for structs with named fields");
    }
}

/// Get `stremio-core` crate alias to use in proc macro.
/// # Errors
/// ```text
/// "Could not find `stremio-core` in `dependencies` or `dev-dependencies` in `stremio-seed-poc/Cargo.toml`!"
/// ```
fn core_ident() -> Result<Ident, String> {
    let in_itself = env::var("CARGO_PKG_NAME").unwrap() == CORE_CRATE_ORIGINAL_NAME;
    let core_crate_name = if in_itself {
        Cow::Borrowed("crate")
    } else {
        // `crate_name` is a `$crate` alternative for proc macros (https://github.com/rust-lang/rust/issues/54363)
        Cow::Owned(crate_name(CORE_CRATE_ORIGINAL_NAME)?)
    };
    Ok(Ident::new(&core_crate_name, Span::call_site()))
}
