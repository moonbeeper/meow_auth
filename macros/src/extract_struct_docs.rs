use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn impl_derive(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let root_docs = get_docs(&input.attrs);
    // println!("root docs: {:?}", root_docs)

    let mut items = Vec::new();

    if let syn::Data::Struct(data) = &input.data {
        match &data.fields {
            syn::Fields::Named(field) => {
                for field in field.named.iter() {
                    let name = field.ident.as_ref().unwrap().to_string();
                    let docs = get_docs(&field.attrs);

                    let ty = &field.ty; // type is a reserved word lol

                    let params = quote! {
                        crate::settings::DocEntry {
                            key: #name,
                            node: &crate::settings::DocNode {
                                docs: &[#(#docs),*],
                                type_docs: &<#ty as crate::settings::HasDocs>::DOCS.docs,
                                children: &<#ty as crate::settings::HasDocs>::DOCS.children,
                            }
                        }
                    };

                    items.push(params);
                }
            }
            _ => {
                panic!("ExtractStructDocs only works on structs with named fields")
            }
        }
    }
    // println!("items: {:#?}", items)

    let expanded = quote! {
        #[automatically_derived]
        impl crate::settings::HasDocs for #name {
            const DOCS: &'static crate::settings::DocNode = &crate::settings::DocNode {
                docs: &[#(#root_docs),*],
                type_docs: &[],
                children: &[#(#items),*]
            }; // YOU MUST BE KIDDING ME THIS HECKING ";" MADE 66 ERRORS THAT I COULD HECKING FIGURE OUT UNTIL NOW I HATE YOU I AHTEEEEEE YOU
        }
    };

    expanded
}

pub fn get_docs(attribute: &[syn::Attribute]) -> Vec<String> {
    let mut docs = Vec::new();
    for attr in attribute {
        // println!("attr: {:#?}", attr);
        // did not know you could chain if let staments like that :o
        if attr.path().is_ident("doc")
            && let syn::Meta::NameValue(name_v) = &attr.meta
            && let syn::Expr::Lit(value) = &name_v.value
            && let syn::Lit::Str(v) = &value.lit
        {
            docs.push(v.value().trim().to_string());
        }
    }
    docs
}
