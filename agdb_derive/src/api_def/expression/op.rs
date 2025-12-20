use crate::api_def::expression;
use crate::api_def::statement::ExpressionContext;
use proc_macro2::TokenStream;
use quote::quote;
use syn::BinOp;
use syn::ExprBinary;
use syn::ExprUnary;

pub(crate) fn parse_assign(assign: &syn::ExprAssign, context: ExpressionContext) -> TokenStream {
    let left = expression::parse_expression(&assign.left, context.inner());
    let right = expression::parse_expression(&assign.right, context.inner());

    quote! {
        {
            ::agdb::api_def::Expression::Assign {
                target: &#left,
                value: &#right,
            }
        }
    }
}

pub(crate) fn parse_binary_op(bin: &ExprBinary, context: ExpressionContext) -> TokenStream {
    let left = expression::parse_expression(&bin.left, context.inner());
    let right = expression::parse_expression(&bin.right, context.inner());
    let op = parse_op(&bin.op);

    quote! {
        ::agdb::api_def::Expression::Binary {
            left: &#left,
            op: #op,
            right: &#right,
        }
    }
}

pub(crate) fn parse_unary_op(un: &ExprUnary, context: ExpressionContext) -> TokenStream {
    let expr = expression::parse_expression(&un.expr, context.inner());
    let op = match &un.op {
        syn::UnOp::Deref(_) => quote! { ::agdb::api_def::Op::Deref },
        syn::UnOp::Not(_) => quote! { ::agdb::api_def::Op::Not },
        syn::UnOp::Neg(_) => quote! { ::agdb::api_def::Op::Neg },
        _ => panic!(
            "{}: Unsupported unary operator: {:?}",
            context.fn_name, un.op
        ),
    };

    quote! {
        ::agdb::api_def::Expression::Unary {
            op: #op,
            expr: &#expr,
        }
    }
}

fn parse_op(op: &BinOp) -> TokenStream {
    match op {
        BinOp::Add(_) => quote! { ::agdb::api_def::Op::Add },
        BinOp::Sub(_) => quote! { ::agdb::api_def::Op::Sub },
        BinOp::Mul(_) => quote! { ::agdb::api_def::Op::Mul },
        BinOp::Div(_) => quote! { ::agdb::api_def::Op::Div },
        BinOp::Rem(_) => quote! { ::agdb::api_def::Op::Rem },
        BinOp::BitXor(_) => quote! { ::agdb::api_def::Op::BitXor },
        BinOp::BitAnd(_) => quote! { ::agdb::api_def::Op::BitAnd },
        BinOp::BitOr(_) => quote! { ::agdb::api_def::Op::BitOr },
        BinOp::Lt(_) => quote! { ::agdb::api_def::Op::Lt },
        BinOp::Gt(_) => quote! { ::agdb::api_def::Op::Gt },
        BinOp::And(_) => quote! { ::agdb::api_def::Op::And },
        BinOp::Or(_) => quote! { ::agdb::api_def::Op::Or },
        BinOp::Shl(_) => quote! { ::agdb::api_def::Op::Shl },
        BinOp::Shr(_) => quote! { ::agdb::api_def::Op::Shr },
        BinOp::Eq(_) => quote! { ::agdb::api_def::Op::Eq },
        BinOp::Le(_) => quote! { ::agdb::api_def::Op::Le },
        BinOp::Ne(_) => quote! { ::agdb::api_def::Op::Ne },
        BinOp::Ge(_) => quote! { ::agdb::api_def::Op::Ge },
        BinOp::AddAssign(_) => quote! { ::agdb::api_def::Op::AddAssign },
        BinOp::SubAssign(_) => quote! { ::agdb::api_def::Op::SubAssign },
        BinOp::MulAssign(_) => quote! { ::agdb::api_def::Op::MulAssign },
        BinOp::DivAssign(_) => quote! { ::agdb::api_def::Op::DivAssign },
        BinOp::RemAssign(_) => quote! { ::agdb::api_def::Op::RemAssign },
        BinOp::BitXorAssign(_) => quote! { ::agdb::api_def::Op::BitXorAssign },
        BinOp::BitAndAssign(_) => quote! { ::agdb::api_def::Op::BitAndAssign },
        BinOp::BitOrAssign(_) => quote! { ::agdb::api_def::Op::BitOrAssign },
        BinOp::ShlAssign(_) => quote! { ::agdb::api_def::Op::ShlAssign },
        BinOp::ShrAssign(_) => quote! { ::agdb::api_def::Op::ShrAssign },
        _ => panic!("Unsupported binary operator"),
    }
}
