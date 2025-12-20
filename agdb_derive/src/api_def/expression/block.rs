use crate::api_def::statement;
use crate::api_def::statement::ExpressionContext;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Block;

pub(crate) fn parse_block(block: &Block, context: ExpressionContext) -> TokenStream {
    let expressions = parse_block_impl(block, context);

    quote! {
        ::agdb::api_def::Expression::Block(&[#(#expressions),*])
    }
}

pub(crate) fn parse_block_impl(block: &Block, context: ExpressionContext) -> Vec<TokenStream> {
    let context = context.inner();
    block
        .stmts
        .iter()
        .enumerate()
        .map(|(i, stmt)| {
            statement::parse_statement(
                stmt,
                if i + 1 == block.stmts.len() {
                    context.last()
                } else {
                    context
                },
            )
        })
        .collect()
}
