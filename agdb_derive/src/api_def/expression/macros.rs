use crate::api_def::expression;
use crate::api_def::expression::path;
use crate::api_def::statement::ExpressionContext;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;
use syn::ExprMacro;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::token::Comma;

pub(crate) fn parse_macros(e: &ExprMacro, context: ExpressionContext) -> TokenStream {
    let name = path::parse_identifier_to_string(&e.mac.path);
    let args: Punctuated<Expr, Comma> = Punctuated::parse_terminated
        .parse2(e.mac.tokens.clone())
        .unwrap_or_default();

    match name.as_str() {
        "vec" => {
            let elements = args
                .iter()
                .map(|arg| expression::parse_expression(arg, context.inner()));
            quote! {
                ::agdb::api_def::Expression::Array(&[#(#elements),*])
            }
        }
        "format" => {
            let (format_string, args) = parse_format_string(args, context);

            quote! {
                ::agdb::api_def::Expression::Format {
                    format_string: #format_string,
                    args: &[#(#args),*],
                }
            }
        }
        _ => {
            panic!("Unsupported macro: {}", name);
        }
    }
}

fn parse_format_string(
    args: Punctuated<Expr, Comma>,
    context: ExpressionContext,
) -> (TokenStream, Vec<TokenStream>) {
    let mut args_iter = args.into_iter();
    let format_string_expr = args_iter.next().unwrap_or_else(|| {
        panic!(
            "{}: format! macro requires at least one argument",
            context.fn_name
        )
    });
    let format_string = extract_format_string(&format_string_expr);
    extract_format_string_with_args(format_string, args_iter, context)
}

fn extract_format_string(e: &Expr) -> String {
    match e {
        Expr::Lit(expr_lit) => {
            if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                lit_str.value()
            } else {
                panic!("First argument to format! must be a string literal");
            }
        }
        _ => panic!("First argument to format! must be a string literal"),
    }
}

fn extract_format_string_with_args(
    format_string: String,
    mut args_iter: impl Iterator<Item = Expr>,
    context: ExpressionContext,
) -> (TokenStream, Vec<TokenStream>) {
    let mut args = Vec::new();
    let mut format_iter = format_string.chars();
    let mut format_str = String::new();

    while let Some(c) = format_iter.next() {
        format_str.push(c);

        if c == '{'
            && let Some(next_char) = format_iter.next()
        {
            if next_char == '{' {
                format_str.push(next_char);
                continue; //escaped brace (double brace)
            }

            format_str.push('}');

            if next_char == '}' {
                let arg = args_iter
                    .next()
                    .unwrap_or_else(|| panic!("{}: not enough args", context.fn_name));
                args.push(expression::parse_expression(&arg, context));
            } else {
                let mut ident = next_char.to_string();

                for nc in format_iter.by_ref() {
                    if nc == '}' {
                        break;
                    }
                    ident.push(nc);
                }

                args.push(quote! {
                    ::agdb::api_def::Expression::Ident(#ident)
                });
            }
        }
    }

    (quote! { stringify!(#format_str) }, args)
}
