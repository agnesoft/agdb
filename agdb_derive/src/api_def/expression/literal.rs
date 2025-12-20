use crate::api_def::statement::ExpressionContext;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Lit;

pub(crate) fn parse_literal(lit: &Lit, _context: ExpressionContext) -> TokenStream {
    match lit {
        Lit::Str(lit_str) => {
            let value = lit_str.value();
            quote! {
                ::agdb::api_def::Expression::Literal(::agdb::api_def::Literal::String(stringify!(#value)))
            }
        }
        Lit::Int(lit_int) => {
            let value = lit_int.base10_parse::<i64>().unwrap();
            quote! {
                ::agdb::api_def::Expression::Literal(::agdb::api_def::Literal::I64(#value))
            }
        }
        Lit::Float(lit_float) => {
            let value = lit_float.base10_parse::<f64>().unwrap();
            quote! {
                ::agdb::api_def::Expression::Literal(::agdb::api_def::Literal::F64(#value))
            }
        }
        Lit::Bool(lit_bool) => {
            let value = lit_bool.value;
            quote! {
                ::agdb::api_def::Expression::Literal(::agdb::api_def::Literal::Bool(#value))
            }
        }
        _ => panic!("Unsupported literal: {:?}", lit),
    }
}
