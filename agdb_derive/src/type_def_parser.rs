pub(crate) mod enum_parser;
pub(crate) mod expression_parser;
pub(crate) mod function_parser;
pub(crate) mod generics_parser;
pub(crate) mod impl_parser;
pub(crate) mod struct_parser;
pub(crate) mod trait_parser;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::DeriveInput;
use syn::Ident;

pub(crate) fn type_def_impl(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as DeriveInput);

    match &input.data {
        syn::Data::Struct(s) => struct_parser::parse_struct(&input, s),
        syn::Data::Enum(e) => enum_parser::parse_enum(&input, e),
        _ => unimplemented!("Only structs and enums are supported for now"),
    }
    .into()
}

pub(crate) fn type_def_impl_impl(item: TokenStream) -> TokenStream {
    let it = item.clone();
    let def: TokenStream2 = type_def_impl(item).into();

    let input = syn::parse_macro_input!(it as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generic, where_clause) = input.generics.split_for_impl();

    quote! {
        #def

        impl #impl_generics ::agdb::type_def::ImplDefinition for #name #ty_generic #where_clause {}
    }
    .into()
}

pub(crate) fn impl_def_impl(item: TokenStream) -> TokenStream {
    let it: TokenStream2 = item.clone().into();
    let def_impl = match syn::parse::<syn::ItemImpl>(item) {
        Ok(input) => impl_parser::parse_impl(&input),
        Err(_) => {
            return crate::compile_error(it, "Only impl blocks are supported").into();
        }
    };

    quote! {
        #it

        #def_impl
    }
    .into()
}

pub(crate) fn trait_def_impl(item: TokenStream) -> TokenStream {
    let it: TokenStream2 = item.clone().into();
    let def_fn = match syn::parse::<syn::ItemTrait>(item) {
        Ok(input) => trait_parser::parse_trait(&input),
        Err(_) => {
            return crate::compile_error(it, "Only traits are supported").into();
        }
    };

    quote! {
        #it

        #def_fn
    }
    .into()
}

pub(crate) fn fn_def_impl(item: TokenStream) -> TokenStream {
    parse_fn_attr_impl(item, quote! { ::agdb::type_def::Type::Function })
}

pub(crate) fn test_def_impl(item: TokenStream) -> TokenStream {
    parse_fn_attr_impl(item, quote! { ::agdb::type_def::Type::Test })
}

fn parse_fn_attr_impl(item: TokenStream, wrapper: TokenStream2) -> TokenStream {
    let it: TokenStream2 = item.clone().into();

    let def_fn = match syn::parse::<syn::ItemFn>(item) {
        Ok(input) => function_parser::parse_function_internal(&input, wrapper),
        Err(_) => {
            return crate::compile_error(it, "Only functions are supported").into();
        }
    };

    quote! {
        #it

        #def_fn
    }
    .into()
}

pub(crate) fn type_def_fn(name: &String) -> TokenStream2 {
    let bound_fn_name = Ident::new(
        &format!("__{name}_type_def"),
        proc_macro2::Span::call_site(),
    );

    quote! { #bound_fn_name }
}
