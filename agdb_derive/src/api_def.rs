pub(crate) mod enum_def;
pub(crate) mod expression;
pub(crate) mod function_def;
pub(crate) mod generics;
pub(crate) mod statement;
pub(crate) mod struct_def;
pub(crate) mod tuple_def;
pub(crate) mod type_def;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::DeriveInput;
use syn::ImplItem;
use syn::ItemImpl;
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
