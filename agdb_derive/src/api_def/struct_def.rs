use super::generics;
use super::type_def;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::FieldsNamed;
use syn::Ident;

pub(crate) fn parse_struct(fields: Option<&FieldsNamed>, input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let generic_names = generics::list_generics(&input.generics);
    let generics = generics::parse_generics(name, &input.generics);
    let fields = parse_named_fields(name, fields, &generic_names);
    let (impl_generics, ty_generic, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::agdb::api_def::TypeDefinition for #name #ty_generic #where_clause {
            fn type_def() -> ::agdb::api_def::Type {
                ::agdb::api_def::Type::Struct(::agdb::api_def::struct_def::Struct {
                    name: stringify!(#name),
                    generics: &[#(#generics),*],
                    fields: &[#(#fields),*],
                    functions: <#name #ty_generic as ::agdb::api_def::ImplDefinition>::functions()
                })
            }
        }
    }
}

pub(crate) fn parse_named_fields(
    name: &Ident,
    fields: Option<&FieldsNamed>,
    generics: &[String],
) -> Vec<TokenStream> {
    if let Some(fields) = fields {
        fields
            .named
            .iter()
            .map(|f| {
                let field_name = f
                    .ident
                    .as_ref()
                    .unwrap_or_else(|| panic!("{name}: Named fields should have an ident"));
                let field_type = type_def::parse_type(&f.ty, generics);
                quote! {
                    ::agdb::api_def::NamedType {
                        name: stringify!(#field_name),
                        ty: Some(#field_type),
                    }
                }
            })
            .collect()
    } else {
        Vec::new()
    }
}
