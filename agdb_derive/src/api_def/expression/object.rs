use proc_macro2::TokenStream;
use quote::quote;
use syn::ExprField;
use syn::ExprStruct;
use syn::FieldValue;
use syn::Member;

use crate::api_def::expression;
use crate::api_def::expression::path;
use crate::api_def::statement::ExpressionContext;

pub(crate) fn parse_field_access(e: &ExprField, context: ExpressionContext) -> TokenStream {
    let base_ts = expression::parse_expression(&e.base, context.inner());
    match &e.member {
        Member::Named(ident) => {
            quote! {
                ::agdb::api_def::Expression::FieldAccess {
                    base: &#base_ts,
                    field: stringify!(#ident),
                }
            }
        }
        Member::Unnamed(index) => {
            let index = index.index;
            quote! {
                ::agdb::api_def::Expression::TupleAccess {
                    base: &#base_ts,
                    index: #index,
                }
            }
        }
    }
}

pub(crate) fn parse_tuple(e: &syn::ExprTuple, context: ExpressionContext) -> TokenStream {
    let elements = e
        .elems
        .iter()
        .map(|elem| expression::parse_expression(elem, context.inner()));
    quote! {
        ::agdb::api_def::Expression::Tuple(&[#(#elements),*])
    }
}

pub(crate) fn parse_struct(e: &ExprStruct, context: ExpressionContext) -> TokenStream {
    let path = path::parse_path(&e.path, context.inner());
    let fields = e
        .fields
        .iter()
        .map(|field| parse_struct_field(field, &context));
    quote! {
        ::agdb::api_def::Expression::Struct {
            name: &#path,
            fields: &[#(#fields),*],
        }
    }
}

fn parse_struct_field(field: &FieldValue, context: &ExpressionContext) -> TokenStream {
    let field_name = match &field.member {
        Member::Named(ident) => ident,
        Member::Unnamed(_) => panic!("Unnamed fields are not supported in struct expressions"),
    };
    let field_value = expression::parse_expression(&field.expr, context.inner());

    quote! {
        (stringify!(#field_name), #field_value)
    }
}
