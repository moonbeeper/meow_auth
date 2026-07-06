use darling::{FromDeriveInput, FromVariant};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::Ident;

#[derive(Debug, FromDeriveInput)]
#[darling(supports(enum_any), attributes(flatten_enum))]
pub struct Input {
    pub ident: syn::Ident,
    pub data: darling::ast::Data<EnumVariant, darling::util::Ignored>,
    #[darling(default)]
    pub utoipa_name: Option<String>,
}

#[derive(Debug, FromVariant, Clone)]
pub struct EnumVariant {
    pub ident: syn::Ident,
    pub fields: darling::ast::Fields<()>,
}

// probably shouldn't be doing these things this way but womp
impl Input {
    pub fn as_derive_input(input: &syn::DeriveInput) -> darling::Result<Self> {
        let this = Self::from_derive_input(input)?;
        Ok(this)
    }
}

impl ToTokens for Input {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let root_ident = &self.ident;
        let new_ident = Ident::new(&format!("{}Flattened", root_ident), root_ident.span());
        let variants: Vec<_> = match &self.data {
            darling::ast::Data::Enum(items) => items.iter().collect(),
            darling::ast::Data::Struct(_) => {
                unreachable!("how, what. it should only be used by enums pf")
            }
        };

        let mut idents: Vec<_> = Vec::new();
        let mut arms: Vec<_> = Vec::new();
        for variant in variants {
            let ident = &variant.ident;
            idents.push(ident);
            let params = if variant.fields.is_unit() {
                quote! {}
            } else if variant.fields.is_tuple() {
                quote! { (..)}
            } else if variant.fields.is_struct() {
                quote! { { .. } }
            } else {
                unreachable!(
                    "somehow enum is nor unit, tuple or struct variant. What abomination is this?"
                )
            };

            arms.push(quote! {
                #root_ident::#ident #params => #new_ident::#ident
            })
        }

        let utoipa_schema = if let Some(name) = &self.utoipa_name {
            let ident = Ident::new(name, Span::call_site());
            quote! {
                #[schema(as = #ident)]
            }
        } else {
            quote! {}
        };

        tokens.extend(quote! {
            #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
            #utoipa_schema
            #[automatically_derived]
            pub enum #new_ident {
                #(#idents,)*
            }

            #[automatically_derived]
            impl From<#root_ident> for #new_ident {
                fn from(value: #root_ident) -> Self {
                    match value {
                        #(#arms,)*
                    }
                }
            }
        });
    }
}
