use super::enum_def;
use super::struct_def;
use super::tuple_def;
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;
use syn::DeriveInput;
use syn::Fields;
use syn::Type;

pub(crate) fn type_def(input: DeriveInput) -> TokenStream {
    match &input.data {
        syn::Data::Struct(s) => match &s.fields {
            Fields::Named(fields) => struct_def::parse_struct(Some(fields), &input),
            Fields::Unnamed(fields) => tuple_def::parse_tuple(Some(fields), &input),
            Fields::Unit => struct_def::parse_struct(None, &input),
        },
        syn::Data::Enum(e) => enum_def::parse_enum(e, &input),
        syn::Data::Union(_) => {
            panic!("{}: Union types are not supported", input.ident);
        }
    }
}

pub(crate) fn parse_type(ty: &Type, list_generics: &[String]) -> TokenStream {
    let ty_str = ty.to_token_stream().to_string();
    if list_generics.contains(&ty_str) {
        quote! { || ::agdb::api_def::Type::Struct(::agdb::api_def::Struct {
            name: stringify!(#ty),
            generics: &[],
            fields: &[],
            functions: &[],
        }) }
    } else {
        quote! { <#ty as ::agdb::api_def::TypeDefinition>::type_def }
    }
}
