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
use syn::punctuated::Punctuated;

pub(crate) fn parse_function(input: &ImplItemFn, impl_generics: &Generics) -> TokenStream {
    let name = &input.sig.ident;
    let mut list_generics = generics::list_generics(impl_generics);
    list_generics.extend(generics::list_generics(&input.sig.generics));
    // Treat `Self` as a special generic-like token to avoid referencing it
    // inside generated const contexts where `Self` is not permitted.
    list_generics.push("Self".to_string());
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
