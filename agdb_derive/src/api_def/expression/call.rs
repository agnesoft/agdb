use crate::api_def::expression;
use crate::api_def::statement::ExpressionContext;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ExprCall;
use syn::ExprMethodCall;

pub(crate) fn parse_call(e: &ExprCall, context: ExpressionContext) -> TokenStream {
    let function = expression::parse_expression(&e.func, context.inner());
    let args = e
        .args
        .iter()
        .map(|arg| expression::parse_expression(arg, context.inner()));

    quote! {
        ::agdb::api_def::Expression::Call {
            recipient: None,
            function: &#function,
            args: &[#(#args),*],
        }
    }
}

pub(crate) fn parse_method_call(e: &ExprMethodCall, context: ExpressionContext) -> TokenStream {
    let recipient = expression::parse_expression(&e.receiver, context.inner());
    let function = &e.method;
    let generics = e
        .turbofish
        .as_ref()
        .map(|gt| {
            gt.args
                .iter()
                .map(|ty| quote! { <#ty as ::agdb::api_def::TypeDefinition>::type_def })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let args = e
        .args
        .iter()
        .map(|arg| expression::parse_expression(arg, context.inner()));

    quote! {
        ::agdb::api_def::Expression::Call {
            recipient: Some(&#recipient),
            function: &::agdb::api_def::Expression::Path {
                ident: stringify!(#function),
                parent: None,
                generics: &[#(#generics),*],
            },
            args: &[#(#args),*],
        }
    }
}
