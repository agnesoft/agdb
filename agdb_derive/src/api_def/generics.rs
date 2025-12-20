use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;
use std::collections::HashMap;
use syn::GenericParam;
use syn::Generics;
use syn::Ident;
use syn::Token;
use syn::TypeParamBound;
use syn::punctuated::Punctuated;

pub fn parse_bounds(
    name: &Ident,
    bounds: &Punctuated<TypeParamBound, Token![+]>,
) -> Vec<TokenStream> {
    bounds
        .iter()
        .filter_map(|st| match st {
            TypeParamBound::Trait(trait_bound) => {
                let trait_name = &trait_bound
                    .path
                    .segments
                    .last()
                    .unwrap_or_else(|| panic!("{name}: Expected trait segment"))
                    .ident;
                let trait_function =
                    Ident::new(&format!("__{}_trait_def", trait_name), trait_name.span());

                Some(quote! { #trait_function })
            }
            TypeParamBound::Lifetime(_) => None,
            TypeParamBound::PreciseCapture(_) => {
                panic!("{name}: PreciseCapture not supported")
            }
            TypeParamBound::Verbatim(_) => panic!("{name}: Verbatim not supported"),
            _ => None,
        })
        .collect()
}

pub fn parse_generics(name: &Ident, generics: &Generics) -> Vec<TokenStream> {
    let where_map = if let Some(where_clause) = &generics.where_clause {
        parse_where_predicates(name, &where_clause.predicates)
    } else {
        HashMap::new()
    };

    parse_generic_params(name, &generics.params, where_map)
}

pub fn list_generics(generics: &Generics) -> Vec<String> {
    generics
        .params
        .iter()
        .filter_map(|param| match param {
            syn::GenericParam::Type(type_param) => Some(type_param.ident.to_string()),
            _ => None,
        })
        .collect()
}

fn parse_where_predicates(
    name: &Ident,
    predicates: &Punctuated<syn::WherePredicate, Token![,]>,
) -> HashMap<String, Vec<TokenStream>> {
    let mut map = HashMap::new();
    predicates.iter().for_each(|pred| match pred {
        syn::WherePredicate::Type(type_pred) => {
            let type_name = &type_pred.bounded_ty;
            let bounds = parse_bounds(name, &type_pred.bounds);
            let name_str = type_name.to_token_stream().to_string();
            map.insert(name_str, bounds);
        }
        syn::WherePredicate::Lifetime(_) => {}
        _ => {}
    });
    map
}

fn parse_generic_params(
    name: &Ident,
    generics: &Punctuated<GenericParam, Token![,]>,
    where_map: HashMap<String, Vec<TokenStream>>,
) -> Vec<TokenStream> {
    generics
        .iter()
        .filter_map(|param| match param {
            syn::GenericParam::Lifetime(_) => None,
            syn::GenericParam::Type(type_param) => {
                let type_name = &type_param.ident;
                let type_name_str = type_name.to_token_stream().to_string();
                let bounds = if let Some(where_bounds) = where_map.get(&type_name_str) {
                    where_bounds
                } else {
                    &parse_bounds(name, &type_param.bounds)
                };

                Some(quote! {
                    ::agdb::api_def::Generic {
                        name: stringify!(#type_name),
                        bounds: &[#(#bounds),*],
                    }
                })
            }
            syn::GenericParam::Const(const_param) => panic!(
                "{name}: Const generic parameters are not supported: {}",
                const_param.ident
            ),
        })
        .collect()
}
