use super::generics;
use super::type_def;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::FieldsUnnamed;

pub(crate) fn parse_tuple(fields: Option<&FieldsUnnamed>, input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let generic_names = generics::list_generics(&input.generics);
    let generics = generics::parse_generics(name, &input.generics);
    let fields = parse_unnamed_fields(fields, &generic_names);
    let (impl_generics, ty_generic, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::agdb::api_def::TypeDefinition for #name #ty_generic #where_clause {
            fn type_def() -> ::agdb::api_def::Type {
                ::agdb::api_def::Type::Tuple(::agdb::api_def::tuple_def::Tuple {
                    name: stringify!(#name),
                    generics: &[#(#generics),*],
                    fields: &[#(#fields),*],
                    functions: <#name #ty_generic as ::agdb::api_def::ImplDefinition>::functions(),
                })
            }
        }
    }
}

pub(crate) fn parse_unnamed_fields(
    fields: Option<&FieldsUnnamed>,
    generics: &[String],
) -> Vec<TokenStream> {
    if let Some(fields) = fields {
        fields
            .unnamed
            .iter()
            .map(|f| {
                let field_type = type_def::parse_type(&f.ty, generics);

                quote! {
                    #field_type
                }
            })
            .collect()
    } else {
        Vec::new()
    }
}
