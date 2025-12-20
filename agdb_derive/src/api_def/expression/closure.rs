use crate::api_def::expression;
use crate::api_def::expression::block;
use crate::api_def::expression::pattern;
use crate::api_def::function_def;
use crate::api_def::statement::ExpressionContext;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;
use syn::ExprClosure;

pub(crate) fn parse_closure(e: &ExprClosure, context: ExpressionContext) -> TokenStream {
    let args: Vec<TokenStream> = e
        .inputs
        .iter()
        .map(|arg| {
            let (name, ty) = pattern::parse_pattern_to_string(arg, context);
            quote! {
            ::agdb::api_def::NamedType {
                name: stringify!(#name),
                ty: #ty,
            }}
        })
        .collect();
    let async_fn = e.asyncness.is_some();
    let ret = function_def::parse_ret(&e.output, &[]);
    let body = parse_body(&e.body, context.inner());

    quote! {
        ::agdb::api_def::Expression::Closure(::agdb::api_def::Function {
            name: "",
            generics: &[],
            args: &[#(#args),*],
            ret: #ret,
            async_fn: #async_fn,
            expressions: &[#(#body),*],
        })
    }
}

fn parse_body(e: &Expr, context: ExpressionContext) -> Vec<TokenStream> {
    match e {
        Expr::Block(body) => block::parse_block_impl(&body.block, context),
        _ => vec![expression::parse_expression(e, context.last())],
    }
}
