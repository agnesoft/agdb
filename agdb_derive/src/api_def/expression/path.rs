use crate::api_def::statement::ExpressionContext;
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;
use syn::GenericArgument;
use syn::Path;
use syn::PathArguments;
use syn::PathSegment;

pub(crate) fn parse_path(path: &Path, _context: ExpressionContext) -> TokenStream {
    let mut iter = path.segments.iter();
    let first = iter.next().expect("path should have at least one segment");
    let first_segment = parse_path_segmenet(first, quote! { None });

    iter.fold(first_segment, |path, segment| {
        parse_path_segmenet(segment, quote! { Some(&#path) })
    })
}

fn parse_path_segmenet(path: &PathSegment, parent: TokenStream) -> TokenStream {
    let ident = &path.ident;

    let generics = match &path.arguments {
        PathArguments::AngleBracketed(args) => {
            args.args
                .iter()
                .filter_map(|ga| {
                    match ga {
                        GenericArgument::Type(ty) => {
                            Some(quote! { <#ty as ::agdb::api_def::TypeDefinition>::type_def })
                        }
                        // Skip lifetimes, const generics, bindings, constraints for now
                        _ => None,
                    }
                })
                .collect::<Vec<_>>()
        }
        PathArguments::Parenthesized(args) => args
            .inputs
            .iter()
            .map(|ty| quote! { <#ty as ::agdb::api_def::TypeDefinition>::type_def })
            .collect::<Vec<_>>(),
        PathArguments::None => Vec::new(),
    };

    quote! {
        ::agdb::api_def::Expression::Path {
            ident: stringify!(#ident),
            parent: #parent,
            generics: &[#(#generics),*],
        }
    }
}

pub(crate) fn parse_identifier_to_string(path: &Path) -> String {
    path.segments
        .last()
        .expect("path should not be empty")
        .to_token_stream()
        .to_string()
}
