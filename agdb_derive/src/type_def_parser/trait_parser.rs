use crate::type_def_parser::function_parser;
use crate::type_def_parser::generics_parser;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::ItemTrait;
use syn::TraitItem;

pub(crate) fn parse_trait(input: &ItemTrait) -> TokenStream2 {
    let name_str = input.ident.to_string();
    let fn_name = crate::type_def_parser::type_def_fn(&name_str);
    let lt_params = generics_parser::parse_lifetime_params(&input.generics);
    let lt_generics = if lt_params.is_empty() {
        quote! {}
    } else {
        quote! { <#(#lt_params),*> }
    };
    let generics = generics_parser::parse_generics(&input.generics);
    let bounds = input
        .supertraits
        .iter()
        .map(generics_parser::parse_type_param_bound);
    let trait_generics = generics_parser::extract_generics(&input.generics);
    let functions = input.items.iter().filter_map(|item| match item {
        TraitItem::Fn(trait_fn) => {
            let fn_generics = generics_parser::extract_generics(&trait_fn.sig.generics);
            let mut combined = Vec::new();
            combined.extend_from_slice(&trait_generics);
            combined.extend_from_slice(&fn_generics);
            Some(function_parser::parse_trait_fn(
                &trait_fn.sig,
                trait_fn.default.as_ref(),
                &combined,
            ))
        }
        _ => None,
    });

    quote! {
        fn #fn_name #lt_generics () -> ::agdb::type_def::Type {
            ::agdb::type_def::Type::Trait(::agdb::type_def::Trait {
                name: #name_str,
                generics: &[#(#generics),*],
                bounds: &[#(#bounds),*],
                functions: &[#(#functions),*],
            })
        }
    }
}
