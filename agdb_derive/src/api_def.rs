pub(crate) mod enum_def;
pub(crate) mod expression;
pub(crate) mod function_def;
pub(crate) mod generics;
pub(crate) mod statement;
pub(crate) mod struct_def;
pub(crate) mod trait_def;
pub(crate) mod tuple_def;
pub(crate) mod type_def;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::DataEnum;
use syn::DeriveInput;
use syn::Generics;
use syn::Ident;
use syn::ImplItem;
use syn::ItemImpl;
use syn::ItemTrait;
use syn::Type;
use syn::parse_macro_input;

pub(crate) fn type_def_impl(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    type_def::type_def(input).into()
}

pub(crate) fn type_def_impl_impl(item: TokenStream) -> TokenStream {
    let it = item.clone();
    let def: TokenStream2 = type_def_impl(item).into();

    let input = parse_macro_input!(it as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generic, where_clause) = input.generics.split_for_impl();

    quote! {
        #def

        impl #impl_generics ::agdb::api_def::ImplDefinition for #name #ty_generic #where_clause {}
    }
    .into()
}

pub(crate) fn trait_def_impl(item: TokenStream) -> TokenStream {
    let trait_tokens: TokenStream2 = item.clone().into();
    let input = parse_macro_input!(item as ItemTrait);
    let trait_def_tokens = trait_def::trait_def(&input);

    quote! {
        #trait_tokens
        #trait_def_tokens
    }
    .into()
}

pub(crate) fn impl_def_impl(item: TokenStream) -> TokenStream {
    let it: TokenStream2 = item.clone().into();
    let impl_block = parse_macro_input!(item as ItemImpl);
    let ty = impl_block.self_ty;
    let funcs = impl_block
        .items
        .iter()
        .filter_map(|i| {
            if let ImplItem::Fn(f) = i {
                Some(function_def::parse_function(f, &impl_block.generics))
            } else {
                None
            }
        })
        .collect::<Vec<TokenStream2>>();
    let funcs_len = funcs.len();

    let (impl_generics, _ty_generic, where_clause) = impl_block.generics.split_for_impl();

    quote! {
        #it

        impl #impl_generics ::agdb::api_def::ImplDefinition for #ty  #where_clause {
            fn functions() -> &'static [::agdb::api_def::Function] {
                const FUNCTIONS: [::agdb::api_def::Function; #funcs_len] = [#(#funcs),*];
                &FUNCTIONS
            }
        }
    }
    .into()
}

// OLD

pub fn api_def_impl(item: TokenStream) -> TokenStream {
    do_api_def(item, true)
}

pub fn api_def(item: TokenStream) -> TokenStream {
    do_api_def(item, false)
}

fn do_api_def(item: TokenStream, has_impl: bool) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = input.ident;
    let generics = input.generics;

    let tokens = if let syn::Data::Struct(data) = input.data {
        let fields_types = data
            .fields
            .iter()
            .map(|f| (f.ident.as_ref(), &f.ty))
            .collect::<Vec<(Option<&Ident>, &Type)>>();

        struct_def(name, generics, fields_types, has_impl)
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
    has_impl: bool,
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

    let impl_def = if !has_impl {
        quote! {
            impl #impl_generics ::agdb::api::ApiFunctions for #name #ty_generics #where_clause {}
        }
    } else {
        quote! {}
    };

    quote! {
        impl #impl_generics ::agdb::api::ApiDefinition for #name #ty_generics #where_clause {
            fn def() -> ::agdb::api::Type {
                ::agdb::api::Type::Struct(::agdb::api::Struct {
                    name: stringify!(#name).to_string(),
                    fields: vec![
                        #(#named_types),*
                    ],
                    functions: <#name #ty_generics as ::agdb::api::ApiFunctions>::functions,
                })
            }
        }

        #impl_def
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
                ::agdb::api::Type::Enum(::agdb::api::Enum {
                    name: stringify!(#name).to_string(),
                    variants: vec![
                        #(#variants),*
                    ],
                })
            }
        }

        impl ::agdb::api::ApiFunctions for #name {}
    }
}
