use super::generics;
use super::statement;
use super::statement::ExpressionContext;
use crate::api_def::type_def;
use proc_macro2::TokenStream;
use quote::quote;
use syn::FnArg;
use syn::Generics;
use syn::Ident;
use syn::ImplItemFn;
use syn::Token;
use syn::TraitItemFn;
use syn::punctuated::Punctuated;

pub fn parse_trait_functions(name: &Ident, items: &[syn::TraitItem]) -> Vec<TokenStream> {
    items
        .iter()
        .filter_map(|item| match item {
            syn::TraitItem::Fn(trait_item_fn) => Some(parse_trait_function(name, trait_item_fn)),
            syn::TraitItem::Type(_) => None, //parsed separately
            syn::TraitItem::Const(trait_item_const) => {
                panic!(
                    "{name}: Trait consts are not supported ('{}')",
                    trait_item_const.ident
                )
            }
            syn::TraitItem::Macro(_) => {
                panic!("{name}: Trait macros are not supported")
            }
            syn::TraitItem::Verbatim(_) => {
                panic!("{name}: Verbatim trait items are not supported")
            }
            _ => panic!("{name}: Unsupported trait item"),
        })
        .collect()
}

fn parse_trait_function(name: &Ident, trait_item_fn: &TraitItemFn) -> TokenStream {
    let fn_name = &trait_item_fn.sig.ident;
    let generics = generics::parse_generics(
        &Ident::new(&format!("{name}__{fn_name}"), name.span()),
        &trait_item_fn.sig.generics,
    );
    let args = parse_trait_function_args(name, &trait_item_fn.sig.inputs);
    let fn_ret = match &trait_item_fn.sig.output {
        syn::ReturnType::Default => quote! { None },
        syn::ReturnType::Type(_, ty) => quote! {
            Some(<#ty as ::agdb::api_def::TypeDefinition>::type_def)
        },
    };
    let async_fn = trait_item_fn.sig.asyncness.is_some();

    quote! { ::agdb::api_def::Function {
            name: stringify!(#fn_name),
            generics: &[#(#generics),*],
            args: &[#(#args),*],
            ret: #fn_ret,
            async_fn: #async_fn,
            expressions: &[],
        }
    }
}

fn parse_trait_function_args(
    name: &Ident,
    args: &Punctuated<FnArg, Token![,]>,
) -> Vec<TokenStream> {
    args.iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_type) => {
                let var_name = match &*pat_type.pat {
                    syn::Pat::Ident(pat_ident) => &pat_ident.ident,
                    _ => panic!("{name}: Unsupported argument pattern"),
                };
                let var_type = &*pat_type.ty;

                Some(quote! {
                    ::agdb::api_def::NamedType {
                        name: stringify!(#var_name),
                        ty: Some(<#var_type as ::agdb::api_def::TypeDefinition>::type_def),
                    }
                })
            }
        })
        .collect()
}

pub(crate) fn parse_function(input: &ImplItemFn, impl_generics: &Generics) -> TokenStream {
    let name = &input.sig.ident;
    let mut list_generics = generics::list_generics(impl_generics);
    list_generics.extend(generics::list_generics(&input.sig.generics));
    let generics = generics::parse_generics(name, &input.sig.generics);
    let args = parse_args(name, &input.sig.inputs, &list_generics);
    let ret = parse_ret(&input.sig.output, &list_generics);
    let async_fn = input.sig.asyncness.is_some();
    let expressions = statement::parse_statements(
        &input.block.stmts,
        ExpressionContext::new(&name.to_string(), &list_generics),
    );

    quote! {
        ::agdb::api_def::Function {
            name: stringify!(#name),
            generics: &[#(#generics),*],
            args: &[#(#args),*],
            ret: #ret,
            async_fn: #async_fn,
            expressions: &[#(#expressions),*],
        }
    }
}

pub(crate) fn parse_ret(output: &syn::ReturnType, generics: &[String]) -> TokenStream {
    match output {
        syn::ReturnType::Default => quote! { None },
        syn::ReturnType::Type(_, ty) => {
            let ty_token = type_def::parse_type(ty, generics);
            quote! { Some(#ty_token) }
        }
    }
}

fn parse_args(
    name: &Ident,
    inputs: &Punctuated<FnArg, Token![,]>,
    generics: &[String],
) -> Vec<TokenStream> {
    let mut args = vec![];

    for input in inputs.iter() {
        if let FnArg::Typed(pat_type) = input
            && let syn::Pat::Ident(pat_ident) = &*pat_type.pat
        {
            let name = &pat_ident.ident;
            let ty = type_def::parse_type(&pat_type.ty, generics);

            args.push(quote! {
                ::agdb::api_def::NamedType {
                    name: stringify!(#name),
                    ty: Some(#ty),
                }
            });
        } else if let FnArg::Receiver(_) = input {
            continue;
        } else {
            panic!(
                "{name}: Unsupported argument type in function definition: {:?}",
                input
            );
        }
    }

    args
}
