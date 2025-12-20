use super::function_def;
use super::generics;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(crate) fn trait_def(input: &syn::ItemTrait) -> TokenStream {
    let name = &input.ident;
    let function_name = format!("__{name}_trait_def");
    let trait_def_fn = Ident::new(&function_name, name.span());
    let bounds = generics::parse_bounds(name, &input.supertraits);
    let generics = generics::parse_generics(name, &input.generics);
    let functions = function_def::parse_trait_functions(name, &input.items);
    let types = parse_trait_types(&input.items);

    quote! {
        #[allow(non_snake_case)]
        fn #trait_def_fn() -> ::agdb::api_def::Trait {
            ::agdb::api_def::Trait {
                name: stringify!(#name),
                generics: &[#(#generics),*],
                bounds: &[#(#bounds),*],
                types: &[#(#types),*],
                functions: &[#(#functions),*],
            }
        }

    }
}

fn parse_trait_types(items: &[syn::TraitItem]) -> Vec<TokenStream> {
    items
        .iter()
        .filter_map(|item| match item {
            syn::TraitItem::Type(trait_item_type) => {
                let type_name = &trait_item_type.ident;
                Some(
                    quote! { || ::agdb::api_def::Type::Struct(::agdb::api_def::Struct {
                        name: stringify!(#type_name),
                        generics: &[],
                        fields: &[],
                        functions: &[],
                    }) },
                )
            }
            _ => None,
        })
        .collect()
}
