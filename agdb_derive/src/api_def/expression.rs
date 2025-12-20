mod array;
mod block;
mod call;
mod closure;
mod conditionals;
mod literal;
mod loops;
mod macros;
mod object;
mod op;
mod path;
pub(crate) mod pattern;

use super::statement::ExpressionContext;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;
use syn::ExprReference;
use syn::ExprTry;

pub(crate) fn parse_expression(e: &Expr, context: ExpressionContext) -> TokenStream {
    let parsed = parse_expression_impl(e, context);

    if context.last && !context.semicolon && is_returnable(e) {
        quote! {
            ::agdb::api_def::Expression::Return(Some(&#parsed))
        }
    } else {
        parsed
    }
}

fn parse_expression_impl(e: &Expr, context: ExpressionContext) -> TokenStream {
    match e {
        Expr::Array(e) => array::parse_array(e, context),
        Expr::Assign(e) => op::parse_assign(e, context),
        Expr::Async(e) => block::parse_block(&e.block, context),
        Expr::Await(e) => parse_await(&e.base, context),
        Expr::Binary(e) => op::parse_binary_op(e, context),
        Expr::Block(e) => block::parse_block(&e.block, context),
        Expr::Break(e) => loops::parse_break(e, context),
        Expr::Call(e) => call::parse_call(e, context),
        Expr::Cast(e) => parse_expression(&e.expr, context),
        Expr::Closure(e) => closure::parse_closure(e, context),
        Expr::Const(e) => block::parse_block(&e.block, context),
        Expr::Continue(e) => loops::parse_continue(e, context),
        Expr::Field(e) => object::parse_field_access(e, context),
        Expr::ForLoop(e) => loops::parse_for_loop(e, context),
        Expr::Group(_) => quote! {}, // group is invisible and ignored
        Expr::If(e) => conditionals::parse_if(e, context),
        Expr::Index(e) => array::parse_index(e, context),
        Expr::Infer(_) => parse_infer(),
        Expr::Let(e) => pattern::parse_let(&e.pat, Some(&e.expr), context),
        Expr::Lit(e) => literal::parse_literal(&e.lit, context),
        Expr::Loop(e) => loops::parse_loop(e, context),
        Expr::Macro(e) => macros::parse_macros(e, context),
        Expr::Match(e) => conditionals::parse_match(e, context),
        Expr::MethodCall(e) => call::parse_method_call(e, context),
        Expr::Paren(e) => parse_expression_impl(&e.expr, context),
        Expr::Path(e) => path::parse_path(&e.path, context),
        Expr::Range(e) => panic!(
            "{}: range expressions are not supported: {e:?}",
            context.fn_name
        ),
        Expr::RawAddr(e) => panic!("{}: raw address are not supported: {e:?}", context.fn_name),
        Expr::Reference(e) => parse_reference(e, context),
        Expr::Repeat(e) => panic!(
            "{}: repeat expressions are not supported: {e:?}",
            context.fn_name
        ),
        Expr::Return(e) => parse_return(&e.expr, context),
        Expr::Struct(e) => object::parse_struct(e, context),
        Expr::Try(e) => parse_try(e, context),
        Expr::TryBlock(e) => block::parse_block(&e.block, context),
        Expr::Tuple(e) => object::parse_tuple(e, context),
        Expr::Unary(e) => op::parse_unary_op(e, context),
        Expr::Unsafe(e) => panic!(
            "{}: unsafe expressions are not supported: {e:?}",
            context.fn_name
        ),
        Expr::Verbatim(e) => panic!(
            "{}: verbatim expressions are not supported: {e:?}",
            context.fn_name
        ),
        Expr::While(e) => loops::parse_while_loop(e, context),
        Expr::Yield(e) => panic!(
            "{}: yield expressions are not supported: {e:?}",
            context.fn_name
        ),
        _ => panic!("Unsupported expression: {}", context.fn_name),
    }
}

fn parse_await(expr: &Expr, context: ExpressionContext) -> TokenStream {
    let expr = parse_expression(expr, context);
    quote! {
        ::agdb::api_def::Expression::Await(&#expr)
    }
}

fn parse_infer() -> TokenStream {
    quote! {
        ::agdb::api_def::Expression::Wild
    }
}

fn parse_reference(reference: &ExprReference, context: ExpressionContext) -> TokenStream {
    let expr = parse_expression(&reference.expr, context);
    quote! {
        ::agdb::api_def::Expression::Reference(&#expr)
    }
}

fn parse_return(expr: &Option<Box<Expr>>, context: ExpressionContext) -> TokenStream {
    if let Some(expr) = expr {
        let parsed = parse_expression(expr, context);
        quote! {
            ::agdb::api_def::Expression::Return(Some(&#parsed))
        }
    } else {
        quote! {
            ::agdb::api_def::Expression::Return(None)
        }
    }
}

fn parse_try(e: &ExprTry, context: ExpressionContext) -> TokenStream {
    let parsed = parse_expression(&e.expr, context);
    quote! {
        ::agdb::api_def::Expression::Try(&#parsed)
    }
}

fn is_returnable(e: &Expr) -> bool {
    !matches!(
        e,
        Expr::Return(_)
            | Expr::Break(_)
            | Expr::Continue(_)
            | Expr::ForLoop(_)
            | Expr::While(_)
            | Expr::If(_)
    )
}
