use crate::type_def_parser::function_parser;
use crate::type_def_parser::generics_parser;
use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::ItemTrait;
use syn::TraitItem;

pub(crate) fn parse_trait(input: &ItemTrait) -> TokenStream2 {
    let name_str = input.ident.to_string();
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

    let def_struct = Ident::new(&format!("{name_str}Def"), Span::call_site());

    quote! {
        #[allow(non_camel_case_types)]
        pub struct #def_struct;

        impl ::agdb::type_def::TypeDefinition for #def_struct {
            fn type_def() -> ::agdb::type_def::Type {
                ::agdb::type_def::Type::Trait(::agdb::type_def::Trait {
                    name: #name_str,
                    generics: &[#(#generics),*],
                    bounds: &[#(#bounds),*],
                    functions: &[#(#functions),*],
                })
            }
        }
    }
}
