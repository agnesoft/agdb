use proc_macro::TokenStream;
use quote::quote;
use syn::DataEnum;
use syn::DeriveInput;
use syn::Generics;
use syn::Ident;
use syn::Type;
use syn::parse_macro_input;

pub fn api_def(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = input.ident;
    let generics = input.generics;

    let tokens = if let syn::Data::Struct(data) = input.data {
        let fields_types = data
            .fields
            .iter()
            .map(|f| (f.ident.as_ref(), &f.ty))
            .collect::<Vec<(Option<&Ident>, &Type)>>();

        struct_def(name, generics, fields_types)
    } else if let syn::Data::Enum(data) = input.data {
        enum_def(name, data)
    } else {
        unimplemented!()
    };

    tokens.into()
}

fn struct_def(
    name: Ident,
    generics: Generics,
    fields_types: Vec<(Option<&Ident>, &Type)>,
) -> proc_macro2::TokenStream {
    let named_types = fields_types.iter().map(|(name, ty)| {
        if let Some(name) = name {
            quote! {
                ::agdb::api::NamedType {
                    name: stringify!(#name),
                    ty: <#ty as ::agdb::api::ApiDefinition>::def,
                }
            }
        } else {
            quote! {
                ::agdb::api::NamedType {
                    name: "",
                    ty: <#ty as ::agdb::api::ApiDefinition>::def,
                }
            }
        }
    });

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::agdb::api::ApiDefinition for #name #ty_generics #where_clause {
            fn def() -> ::agdb::api::Type {
                static STATIC: ::std::sync::OnceLock<::agdb::api::Struct> = ::std::sync::OnceLock::new();
                ::agdb::api::Type::Struct(STATIC.get_or_init(|| ::agdb::api::Struct {
                    name: stringify!(#name),
                    fields: vec![
                        #(#named_types),*
                    ]
                }))
            }
        }
    }
}

fn enum_def(name: Ident, enum_data: DataEnum) -> proc_macro2::TokenStream {
    let variants = enum_data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        if let Some(field) = variant.fields.iter().next() {
            let ty = &field.ty;

            quote! {
                ::agdb::api::NamedType {
                    name: stringify!(#variant_name),
                    ty: <#ty as ::agdb::api::ApiDefinition>::def,
                }
            }
        } else {
            quote! {
                ::agdb::api::NamedType {
                    name: stringify!(#variant_name),
                    ty: || ::agdb::api::Type::None,
                }
            }
        }
    });

    quote! {
        impl ::agdb::api::ApiDefinition for #name {
            fn def() -> ::agdb::api::Type {
                static STATIC: ::std::sync::OnceLock<::agdb::api::Enum> = ::std::sync::OnceLock::new();
                ::agdb::api::Type::Enum(STATIC.get_or_init(|| ::agdb::api::Enum {
                    name: stringify!(#name),
                    variants: vec![
                        #(#variants),*
                    ],
                }))
            }
        }
    }
}
