use crate::type_def_parser::generics_parser;
use crate::type_def_parser::generics_parser::Generic;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::FnArg;
use syn::ItemFn;
use syn::Pat;
use syn::ReturnType;
use syn::punctuated::Punctuated;
use syn::token::Comma;

pub(crate) fn parse_function(input: &ItemFn) -> TokenStream2 {
    let name_str = input.sig.ident.to_string();
    let fn_name = crate::type_def_parser::type_def_fn(&name_str);
    let current_generics = generics_parser::extract_generics(&input.sig.generics);
    let generics = generics_parser::parse_generics(&input.sig.generics);
    let args = parse_args(&input.sig.inputs, &current_generics);
    let ret = parse_ret(&input.sig.output, &current_generics);
    let async_fn = input.sig.asyncness.is_some();
    let body = crate::type_def_parser::expression_parser::parse_block_stmts(
        &input.block,
        &current_generics,
    );
    let lt_params = generics_parser::parse_lifetime_params(&input.sig.generics);
    let lt_generics = if lt_params.is_empty() {
        quote! {}
    } else {
        quote! { <#(#lt_params),*> }
    };
    let name = &input.sig.ident;

    quote! {
        fn #fn_name #lt_generics () -> ::agdb::type_def::Type {
            ::agdb::type_def::Type::Function(::agdb::type_def::Function {
                name: stringify!(#name),
                generics: &[#(#generics),*],
                args: &[#(#args),*],
                ret: #ret,
                async_fn: #async_fn,
                body: &[#(#body),*],
            })
        }
    }
}

pub(crate) fn parse_signature(sig: &syn::Signature) -> TokenStream2 {
    let name = &sig.ident;
    let current_generics = generics_parser::extract_generics(&sig.generics);
    let generics = generics_parser::parse_generics(&sig.generics);
    let args = parse_args(&sig.inputs, &current_generics);
    let ret = parse_ret(&sig.output, &current_generics);
    let async_fn = sig.asyncness.is_some();

    quote! {
        ::agdb::type_def::Function {
            name: stringify!(#name),
            generics: &[#(#generics),*],
            args: &[#(#args),*],
            ret: #ret,
            async_fn: #async_fn,
            body: &[],
        }
    }
}

pub(crate) fn parse_trait_fn(
    sig: &syn::Signature,
    default_block: Option<&syn::Block>,
    current_generics: &[Generic],
) -> TokenStream2 {
    let name = &sig.ident;
    let generics = generics_parser::parse_generics(&sig.generics);
    let args = parse_args(&sig.inputs, current_generics);
    let ret = parse_ret(&sig.output, current_generics);
    let async_fn = sig.asyncness.is_some();
    let body = default_block
        .map(|block| {
            crate::type_def_parser::expression_parser::parse_block_stmts(block, current_generics)
        })
        .unwrap_or_default();

    quote! {
        ::agdb::type_def::Function {
            name: stringify!(#name),
            generics: &[#(#generics),*],
            args: &[#(#args),*],
            ret: #ret,
            async_fn: #async_fn,
            body: &[#(#body),*],
        }
    }
}

fn parse_args(args: &Punctuated<FnArg, Comma>, generics: &[Generic]) -> Vec<TokenStream2> {
    args.iter()
        .map(|arg| match arg {
            FnArg::Typed(pat_ty) => {
                let name = match pat_ty.pat.as_ref() {
                    Pat::Ident(pat_ident) => {
                        let ident = &pat_ident.ident;
                        quote! { stringify!(#ident) }
                    }
                    _ => quote! { "" },
                };
                let ty_def = generics_parser::parse_type(&pat_ty.ty, generics);

                quote! {
                    ::agdb::type_def::Variable {
                        name: #name,
                        ty: Some(#ty_def),
                    }
                }
            }
            FnArg::Receiver(rec) => {
                if let Some(_token) = rec.colon_token {
                    let ty = &rec.ty;
                    quote! {
                        ::agdb::type_def::Variable {
                            name: "self",
                            ty: Some(<#ty as ::agdb::type_def::TypeDefinition>::type_def),
                        }
                    }
                } else {
                    let mutable = rec.mutability.is_some();

                    let ty = if let Some((_, lt_opt)) = &rec.reference {
                        let lifetime = if let Some(lt) = lt_opt {
                            let lt_str = lt.ident.to_string();
                            quote! { Some(#lt_str) }
                        } else {
                            quote! { None }
                        };
                        quote! {
                            || ::agdb::type_def::Type::Reference(::agdb::type_def::Reference {
                                mutable: #mutable,
                                lifetime: #lifetime,
                                ty: || ::agdb::type_def::Type::SelfType(#mutable),
                            })
                        }
                    } else {
                        quote! { || ::agdb::type_def::Type::SelfType(#mutable) }
                    };
                    quote! {
                        ::agdb::type_def::Variable {
                            name: "self",
                            ty: Some(#ty),
                        }
                    }
                }
            }
        })
        .collect()
}

fn parse_ret(ret: &ReturnType, generics: &[Generic]) -> TokenStream2 {
    match ret {
        ReturnType::Default => quote! { <() as ::agdb::type_def::TypeDefinition>::type_def },
        ReturnType::Type(_, ty) => generics_parser::parse_type(ty, generics),
    }
}
