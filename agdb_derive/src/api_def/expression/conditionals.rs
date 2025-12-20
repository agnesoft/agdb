use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;
use syn::ExprIf;
use syn::ExprMatch;
use syn::Pat;
use syn::PatOr;

use crate::api_def::expression;
use crate::api_def::expression::block;
use crate::api_def::expression::pattern;
use crate::api_def::statement::ExpressionContext;

pub(crate) fn parse_if(if_expr: &ExprIf, context: ExpressionContext) -> TokenStream {
    let condition = expression::parse_expression(&if_expr.cond, context.inner());
    let then_branch = block::parse_block(&if_expr.then_branch, context.inner());

    let else_branch = if let Some((_, else_expr)) = &if_expr.else_branch {
        let else_tokens = match else_expr.as_ref() {
            Expr::If(else_if) => parse_if(else_if, context.inner()),
            Expr::Block(else_block) => block::parse_block(&else_block.block, context.inner()),
            _ => panic!(
                "{} Unsupported else branch in if expression",
                context.fn_name
            ),
        };
        quote! { Some(&#else_tokens) }
    } else {
        quote! { None }
    };

    quote! {
        ::agdb::api_def::Expression::If {
            condition: &#condition,
            then_branch: &#then_branch,
            else_branch: #else_branch,
        }
    }
}

pub(crate) fn parse_match(match_expr: &ExprMatch, context: ExpressionContext) -> TokenStream {
    let subject = expression::parse_expression(&match_expr.expr, context.inner());
    let mut branches = Vec::new();
    let mut else_branch: Option<TokenStream> = None;

    for arm in &match_expr.arms {
        if is_wild(&arm.pat) {
            else_branch = Some(parse_match_arm_body(&arm.body, context.inner()));
        } else {
            let condition = parse_match_condition(&subject, &arm.pat, context.inner());
            let condition_with_guard = if let Some(guard) = &arm.guard {
                let guard_expr = expression::parse_expression(&guard.1, context.inner());
                quote! {
                    ::agdb::api_def::Expression::Binary {
                        left: &#condition,
                        op: ::agdb::api_def::Op::And,
                        right: &#guard_expr,
                    }
                }
            } else {
                condition
            };
            let body = parse_match_arm_body(&arm.body, context.inner());
            branches.push((condition_with_guard, body));
        }
    }

    if branches.is_empty() {
        else_branch.expect("Match expression must have at least one arm")
    } else {
        branches
            .iter()
            .rev()
            .fold(else_branch, |else_branch, (cond, body)| {
                let else_condition = if let Some(else_branch) = else_branch {
                    else_branch
                } else {
                    quote! {
                        ::agdb::api_def::Expression::Block(&[])
                    }
                };

                Some(quote! {
                    ::agdb::api_def::Expression::If {
                        condition: &#cond,
                        then_branch: &#body,
                        else_branch: Some(&#else_condition),
                    }
                })
            })
            .expect("At least one match arm present")
    }
}

fn is_wild(pat: &Pat) -> bool {
    matches!(pat, Pat::Wild(_))
}

fn parse_match_arm_body(body: &Expr, context: ExpressionContext) -> TokenStream {
    match body {
        Expr::Block(b) => block::parse_block(&b.block, context.inner()),

        // Match `()` to an empty block
        Expr::Tuple(t) if t.elems.is_empty() => {
            quote! { &::agdb::api_def::Expression::Block(&[]) }
        }
        // For other expressions, wrap in a block
        expr => {
            let inner = expression::parse_expression(expr, context.inner());
            quote! { &::agdb::api_def::Expression::Block(&[#inner]) }
        }
    }
}

fn parse_match_condition(
    subject: &TokenStream,
    pat: &Pat,
    context: ExpressionContext,
) -> TokenStream {
    if let Pat::Or(p) = pat {
        return match_or(subject, p, context);
    }

    let rhs = pattern::parse_pattern(pat, context).0;

    quote! {
        ::agdb::api_def::Expression::Binary {
            left: &#subject,
            op: ::agdb::api_def::Op::Eq,
            right: &#rhs,
        }
    }
}

fn match_or(subject: &TokenStream, pat_or: &PatOr, context: ExpressionContext) -> TokenStream {
    let mut conds: Vec<TokenStream> = Vec::new();
    for subpat in pat_or.cases.iter() {
        conds.push(parse_match_condition(subject, subpat, context.inner()));
    }
    let mut iter = conds.into_iter();
    let first = iter.next().expect("Or pattern cannot be without cases");
    iter.fold(first, |acc, next| {
        quote! {
            ::agdb::api_def::Expression::Binary {
                left: &#acc,
                op: ::agdb::api_def::Op::Or,
                right: &#next,
            }
        }
    })
}
