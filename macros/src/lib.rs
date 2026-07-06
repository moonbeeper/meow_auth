use proc_macro::TokenStream;
use quote::ToTokens as _;
use syn::{DeriveInput, parse_macro_input};

mod flatten_enum;

/// flattens an enum that has variants with tuples or structs into a new enum
/// with the same variants BUT without those same tuple or structs.
///
/// Its useful for when you want to use an enum with Utoipa's openapi.
#[proc_macro_derive(FlattenEnum, attributes(flatten_enum))]
pub fn flatten_enum(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    let input = match flatten_enum::Input::as_derive_input(&derive_input) {
        Ok(parsed) => parsed,
        Err(err) => return err.write_errors().into(),
    };

    TokenStream::from(input.into_token_stream())
}
