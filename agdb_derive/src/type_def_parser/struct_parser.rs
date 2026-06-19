use crate::type_def_parser::generics_parser;
use crate::type_def_parser::generics_parser::Generic;
use crate::type_def_parser::impl_parser;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::DataStruct;
use syn::DeriveInput;
use syn::Fields;

pub(crate) fn parse_struct(input: &DeriveInput, s: &DataStruct) -> TokenStream2 {
    let name = &input.ident;
    let current_generics = generics_parser::extract_generics(&input.generics);
    let generics = generics_parser::parse_generics(&input.generics);
    let fields = parse_fields(&s.fields, &current_generics);
    let (impl_generics, ty_generic, where_clause) = input.generics.split_for_impl();

    let impl_names = impl_parser::parse_type_def_impls(&input.attrs);
    let impl_defs_method =
        impl_parser::generate_impl_defs_method(&impl_names, &name.to_string(), &input.generics);

    quote! {
        impl #impl_generics ::agdb::type_def::TypeDefinition for #name #ty_generic #where_clause {
            fn type_def() -> ::agdb::type_def::Type {
                ::agdb::type_def::Type::Struct(::agdb::type_def::Struct {
                    name: stringify!(#name).to_owned(),
                    generics: vec![#(#generics),*],
                    fields: vec![#(#fields),*],
                    impl_defs: Self::impl_defs,
                })
            }
            #impl_defs_method
        }
    }
}

fn parse_fields(fields: &Fields, generics: &[Generic]) -> Vec<TokenStream2> {
    fields
        .iter()
        .map(|f| {
            let name = f
                .ident
                .as_ref()
                .map(|ident| quote! { stringify!(#ident).to_owned() })
                .unwrap_or_else(|| quote! { String::new() });
            let ty_def = generics_parser::parse_type(&f.ty, generics);
            quote! {
                ::agdb::type_def::Variable {
                    name: #name,
                    ty: Some(#ty_def),
                }
            }
        })
        .collect::<Vec<_>>()
}
