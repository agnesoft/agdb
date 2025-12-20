use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;
use syn::Pat;
use syn::PatIdent;
use syn::PatOr;
use syn::PatSlice;
use syn::PatTuple;
use syn::PatTupleStruct;

use crate::api_def::expression;
use crate::api_def::expression::literal;
use crate::api_def::expression::path;
use crate::api_def::statement::ExpressionContext;
use crate::api_def::type_def;

pub(crate) fn parse_let(pat: &Pat, init: Option<&Expr>, context: ExpressionContext) -> TokenStream {
    let (name, ty) = parse_pattern(pat, context.inner());
    let value = if let Some(init) = init {
        let expr = expression::parse_expression(init, context.inner());
        quote! { Some(&#expr) }
    } else {
        quote! { None }
    };

    quote! {
        ::agdb::api_def::Expression::Let {
            name: &#name,
            ty: #ty,
            value: #value,
        }
    }
}

pub(crate) fn parse_pattern(pat: &Pat, context: ExpressionContext) -> (TokenStream, TokenStream) {
    match pat {
        Pat::Ident(p) => ident(p),
        Pat::Lit(p) => literal(context, p),
        Pat::Type(p) => typed(p, context),
        Pat::Or(p) => or(p, context),
        Pat::Paren(p) => parse_pattern(&p.pat, context),
        Pat::Path(p) => path(p, context),
        Pat::Reference(p) => parse_pattern(&p.pat, context),
        Pat::Slice(p) => slice(p, context),
        Pat::Struct(p) => struct_pattern(p, context),
        Pat::Tuple(p) => tuple(p, context),
        Pat::TupleStruct(p) => tuple_struct(p, context),
        Pat::Wild(_) => wild(),
        // 1..=10
        Pat::Range(_) //todo?
        // Tuple(expr, expr2, ...)
        | Pat::Rest(_) //todo?
        | Pat::Const(_)
        | Pat::Macro(_)
        | Pat::Verbatim(_)
        | _ => panic!("Unsupported pattern in {}: {:?}", context.fn_name, pat),
    }
}

fn literal(context: ExpressionContext<'_>, p: &syn::PatLit) -> (TokenStream, TokenStream) {
    let lit_expr = literal::parse_literal(&p.lit, context);
    (
        quote! {
            #lit_expr
        },
        quote! {
            None
        },
    )
}

fn tuple_struct(p: &PatTupleStruct, context: ExpressionContext) -> (TokenStream, TokenStream) {
    let path_expr = path::parse_path(&p.path, context.inner());
    let elems = p.elems.iter().map(|elem| parse_pattern(elem, context).0);
    (
        quote! {
            ::agdb::api_def::Expression::TupleStruct {
                name: &#path_expr,
                expressions: &[#(#elems),*],
            }
        },
        quote! {
            None
        },
    )
}

fn tuple(p: &PatTuple, context: ExpressionContext) -> (TokenStream, TokenStream) {
    let elems = p.elems.iter().map(|elem| parse_pattern(elem, context).0);
    (
        quote! {
            ::agdb::api_def::Expression::Tuple(&[#(#elems),*])
        },
        quote! {
            None
        },
    )
}

fn struct_pattern(p: &syn::PatStruct, context: ExpressionContext) -> (TokenStream, TokenStream) {
    let path_expr = path::parse_path(&p.path, context.inner());
    let fields = p.fields.iter().map(|field| {
        let (field_name, _) = parse_pattern(&field.pat, context.inner());
        quote! {
            #field_name
        }
    });
    (
        quote! {
            ::agdb::api_def::Expression::StructPattern {
                name: &#path_expr,
                fields: &[#(#fields),*],
            }
        },
        quote! {
            None
        },
    )
}

fn slice(p: &PatSlice, context: ExpressionContext) -> (TokenStream, TokenStream) {
    let elems = p.elems.iter().map(|elem| parse_pattern(elem, context).0);
    (
        quote! {
            ::agdb::api_def::Expression::Array(&[#(#elems),*])
        },
        quote! {
            None
        },
    )
}

fn path(p: &syn::PatPath, context: ExpressionContext) -> (TokenStream, TokenStream) {
    let path_str = path::parse_path(&p.path, context.inner());
    (
        quote! {
            #path_str
        },
        quote! {
            None
        },
    )
}

fn wild() -> (TokenStream, TokenStream) {
    (
        quote! {
            ::agdb::api_def::Expression::Wild
        },
        quote! {
            None
        },
    )
}

fn typed(pat_type: &syn::PatType, context: ExpressionContext) -> (TokenStream, TokenStream) {
    let (name, _) = parse_pattern(&pat_type.pat, context);
    let ty = type_def::parse_type(&pat_type.ty, context.generics);
    (
        name,
        quote! {
            Some(#ty)
        },
    )
}

fn ident(pat_ident: &PatIdent) -> (TokenStream, TokenStream) {
    let name = &pat_ident.ident;
    (
        quote! {
            ::agdb::api_def::Expression::Ident(stringify!(#name))
        },
        quote! {
            None
        },
    )
}

fn or(pat_or: &PatOr, context: ExpressionContext) -> (TokenStream, TokenStream) {
    let mut conds: Vec<TokenStream> = Vec::new();
    for subpat in pat_or.cases.iter() {
        conds.push(parse_pattern(subpat, context.inner()).0);
    }
    let mut iter = conds.into_iter();
    let first = iter.next().expect("Or pattern cannot be without cases");
    (
        iter.fold(first, |acc, next| {
            quote! {
                ::agdb::api_def::Expression::Binary {
                    left: &#acc,
                    op: ::agdb::api_def::Op::Or,
                    right: &#next,
                }
            }
        }),
        quote! {
            None
        },
    )
}

pub(crate) fn parse_pattern_to_string(
    pat: &Pat,
    context: ExpressionContext,
) -> (TokenStream, TokenStream) {
    match pat {
        Pat::Ident(pat_ident) => {
            let name = &pat_ident.ident;
            (
                quote! {
                   #name
                },
                quote! {
                    None
                },
            )
        }
        Pat::Type(pat_type) => {
            let (name, _) = parse_pattern_to_string(&pat_type.pat, context);
            let ty = type_def::parse_type(&pat_type.ty, context.generics);
            (
                name,
                quote! {
                    Some(#ty)
                },
            )
        }
        Pat::Wild(_) => (
            quote! {
                "_"
            },
            quote! {
                None
            },
        ),
        _ => panic!(
            "Unsupported pattern to string in {}: {:?}",
            context.fn_name, pat
        ),
    }
}
