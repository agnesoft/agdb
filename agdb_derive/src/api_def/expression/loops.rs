use crate::api_def::expression;
use crate::api_def::expression::block;
use crate::api_def::expression::pattern;
use crate::api_def::statement::ExpressionContext;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ExprBreak;
use syn::ExprContinue;
use syn::ExprWhile;

pub(crate) fn parse_for_loop(
    for_loop: &syn::ExprForLoop,
    context: ExpressionContext,
) -> TokenStream {
    let (pattern, _) = pattern::parse_pattern(&for_loop.pat, context.inner());
    let iterable = expression::parse_expression(&for_loop.expr, context.inner());
    let body = block::parse_block(&for_loop.body, context.inner());

    quote! {
        ::agdb::api_def::Expression::For {
            pattern: &#pattern,
            iterable: &#iterable,
            body: &#body,
        }
    }
}

pub(crate) fn parse_break(break_expr: &ExprBreak, context: ExpressionContext) -> TokenStream {
    if break_expr.expr.is_some() {
        panic!("{} Break with value is not supported", context.fn_name);
    }

    quote! {
        ::agdb::api_def::Expression::Break {}
    }
}

pub(crate) fn parse_loop(loop_expr: &syn::ExprLoop, context: ExpressionContext) -> TokenStream {
    let body = block::parse_block(&loop_expr.body, context.inner());

    quote! {
        ::agdb::api_def::Expression::While {
            condition: &::agdb::api_def::Expression::Literal(::agdb::api_def::Literal::Bool(true)),
            body: &#body,
        }
    }
}

pub(crate) fn parse_while_loop(while_expr: &ExprWhile, context: ExpressionContext) -> TokenStream {
    let condition = expression::parse_expression(&while_expr.cond, context.inner());
    let body = block::parse_block(&while_expr.body, context.inner());

    quote! {
        ::agdb::api_def::Expression::While {
            condition: &#condition,
            body: &#body,
        }
    }
}

pub(crate) fn parse_continue(
    _continue_expr: &ExprContinue,
    _context: ExpressionContext,
) -> TokenStream {
    quote! {
        ::agdb::api_def::Expression::Continue {}
    }
}
