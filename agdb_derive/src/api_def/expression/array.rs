use crate::api_def::expression;
use crate::api_def::statement::ExpressionContext;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ExprArray;
use syn::ExprIndex;

pub(crate) fn parse_array(ar: &ExprArray, context: ExpressionContext) -> TokenStream {
    let elements = ar
        .elems
        .iter()
        .map(|elem| expression::parse_expression(elem, context.inner()));
    quote! {
        ::agdb::api_def::Expression::Array(&[#(#elements),*])
    }
}

pub(crate) fn parse_index(e: &ExprIndex, context: ExpressionContext) -> TokenStream {
    let expr = expression::parse_expression(&e.expr, context.inner());
    let index = expression::parse_expression(&e.index, context.inner());
    quote! {
        ::agdb::api_def::Expression::Index {
            base: &#expr,
            index: &#index,
        }
    }
}
