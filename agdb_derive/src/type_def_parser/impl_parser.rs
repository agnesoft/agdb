use crate::type_def_parser::function_parser;
use crate::type_def_parser::generics_parser;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::ImplItem;

pub(crate) fn parse_impl(input: &syn::ItemImpl) -> TokenStream2 {
    let ty = &input.self_ty;
    let generics = generics_parser::parse_generics(&input.generics);
    let (impl_generics, _, where_clause) = input.generics.split_for_impl();

    let trait_part = if let Some((_, path, _)) = &input.trait_ {
        let trait_name = path.segments.last().unwrap().ident.to_string();
        quote! {
            Some(|| ::agdb::type_def::Type::Trait(::agdb::type_def::Trait {
                name: #trait_name,
                generics: &[],
                bounds: &[],
                functions: &[],
            }))
        }
    } else {
        quote! { None }
    };

    let methods = input
        .items
        .iter()
        .map(|item| match item {
            ImplItem::Fn(impl_fn) => function_parser::parse_signature(&impl_fn.sig),
            _ => panic!("Only function items are supported in impl blocks"),
        })
        .collect::<Vec<_>>();

    quote! {
        impl #impl_generics ::agdb::type_def::ImplDefinition for #ty #where_clause {
            fn impl_def() -> ::agdb::type_def::Impl {
                ::agdb::type_def::Impl {
                    name: stringify!(#ty),
                    generics: &[#(#generics),*],
                    trait_: #trait_part,
                    ty: <#ty as ::agdb::type_def::TypeDefinition>::type_def,
                    functions: &[#(#methods),*],
                }
            }
        }
    }
}
