pub(crate) mod enum_parser;
pub(crate) mod expression_parser;
pub(crate) mod function_parser;
pub(crate) mod generics_parser;
pub(crate) mod impl_parser;
pub(crate) mod struct_parser;
pub(crate) mod trait_parser;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::DeriveInput;

pub(crate) fn type_def_impl(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as DeriveInput);

    match &input.data {
        syn::Data::Struct(s) => struct_parser::parse_struct(&input, s),
        syn::Data::Enum(e) => enum_parser::parse_enum(&input, e),
        _ => crate::compile_error(&input.ident, "Only structs and enums are supported"),
    }
    .into()
}

pub(crate) fn impl_def_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr;
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

pub(crate) fn static_def_impl(item: TokenStream) -> TokenStream {
    let it: TokenStream2 = item.clone().into();

    if let Ok(input) = syn::parse::<syn::ItemConst>(item.clone()) {
        let name_str = input.ident.to_string();
        let fn_name = type_def_fn(&name_str);
        let ty = generics_parser::parse_type(&input.ty, &[]);
        let value = expression_parser::parse_expression(&input.expr, &[]);
        let name = &input.ident;

        quote! {
            #it

            fn #fn_name() -> ::agdb::type_def::Type {
                ::agdb::type_def::Type::Static(::agdb::type_def::Static {
                    name: stringify!(#name).to_owned(),
                    ty: #ty,
                    value: vec![#value],
                })
            }
        }
        .into()
    } else if let Ok(input) = syn::parse::<syn::ItemStatic>(item.clone()) {
        let name_str = input.ident.to_string();
        let fn_name = type_def_fn(&name_str);
        let ty = generics_parser::parse_type(&input.ty, &[]);
        let value = expression_parser::parse_expression(&input.expr, &[]);
        let name = &input.ident;

        quote! {
            #it

            fn #fn_name() -> ::agdb::type_def::Type {
                ::agdb::type_def::Type::Static(::agdb::type_def::Static {
                    name: stringify!(#name).to_owned(),
                    ty: #ty,
                    value: vec![#value],
                })
            }
        }
        .into()
    } else {
        crate::compile_error(it, "Only const & static items are supported").into()
    }
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
