use crate::type_def_parser::generics_parser;
use crate::type_def_parser::generics_parser::Generic;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use quote::quote;
use syn::BinOp;
use syn::Block;
use syn::Expr;
use syn::ExprArray;
use syn::ExprBinary;
use syn::ExprCall;
use syn::ExprClosure;
use syn::ExprField;
use syn::ExprIndex;
use syn::ExprMacro;
use syn::ExprMethodCall;
use syn::ExprRange;
use syn::ExprReference;
use syn::ExprStruct;
use syn::ExprTry;
use syn::ExprUnary;
use syn::FieldValue;
use syn::GenericArgument;
use syn::Lit;
use syn::Member;
use syn::Pat;
use syn::PatOr;
use syn::Path;
use syn::PathArguments;
use syn::PathSegment;
use syn::ReturnType;
use syn::Stmt;

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Parse a block into a list of expression token-streams (one per statement).
pub(crate) fn parse_block_stmts(block: &Block, generics: &[Generic]) -> Vec<TokenStream2> {
    block
        .stmts
        .iter()
        .enumerate()
        .map(|(i, stmt)| {
            let last = i + 1 == block.stmts.len();
            parse_statement(stmt, generics, last)
        })
        .collect()
}

/// Parse a block into an `::agdb::type_def::Expression::Block(...)`.
pub(crate) fn parse_block(block: &Block, generics: &[Generic]) -> TokenStream2 {
    let expressions = parse_block_stmts(block, generics);
    quote! {
        ::agdb::type_def::Expression::Block(&[#(#expressions),*])
    }
}

/// Parse a single `syn::Expr` into a `TokenStream2` that constructs an
/// `::agdb::type_def::Expression`.
pub(crate) fn parse_expression(e: &Expr, generics: &[Generic]) -> TokenStream2 {
    match e {
        Expr::Array(e) => parse_array(e, generics),
        Expr::Assign(e) => parse_assign(e, generics),
        Expr::Async(e) => parse_block(&e.block, generics),
        Expr::Await(e) => parse_await(e, generics),
        Expr::Binary(e) => parse_binary(e, generics),
        Expr::Block(e) => parse_block(&e.block, generics),
        Expr::Break(_) => quote! { ::agdb::type_def::Expression::Break },
        Expr::Call(e) => parse_call(e, generics),
        Expr::Cast(e) => parse_expression(&e.expr, generics),
        Expr::Closure(e) => parse_closure(e, generics),
        Expr::Const(e) => parse_block(&e.block, generics),
        Expr::Continue(_) => quote! { ::agdb::type_def::Expression::Continue },
        Expr::Field(e) => parse_field_access(e, generics),
        Expr::ForLoop(e) => parse_for_loop(e, generics),
        Expr::Group(e) => parse_expression(&e.expr, generics),
        Expr::If(e) => parse_if(e, generics),
        Expr::Index(e) => parse_index(e, generics),
        Expr::Infer(_) => quote! { ::agdb::type_def::Expression::Wild },
        Expr::Let(e) => parse_let_expr(e, generics),
        Expr::Lit(e) => parse_literal(&e.lit),
        Expr::Loop(e) => parse_loop(e, generics),
        Expr::Macro(e) => parse_macro(e, generics),
        Expr::Match(e) => parse_match(e, generics),
        Expr::MethodCall(e) => parse_method_call(e, generics),
        Expr::Paren(e) => parse_expression(&e.expr, generics),
        Expr::Path(e) => parse_path(&e.path),
        Expr::Range(e) => parse_range(e, generics),
        Expr::Reference(e) => parse_reference(e, generics),
        Expr::Return(e) => parse_return(e, generics),
        Expr::Struct(e) => parse_struct(e, generics),
        Expr::Try(e) => parse_try(e, generics),
        Expr::Tuple(e) => parse_tuple(e, generics),
        Expr::Unary(e) => parse_unary(e, generics),
        Expr::While(e) => parse_while(e, generics),
        _ => crate::compile_error(
            e,
            format!("Unsupported expression: {}", e.to_token_stream()),
        ),
    }
}

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

fn parse_statement(stmt: &Stmt, generics: &[Generic], last: bool) -> TokenStream2 {
    match stmt {
        Stmt::Local(local) => parse_local(local, generics),
        Stmt::Expr(expr, semi) => {
            let parsed = parse_expression(expr, generics);
            if last && semi.is_none() && is_returnable(expr) {
                quote! { ::agdb::type_def::Expression::Return(Some(&#parsed)) }
            } else {
                parsed
            }
        }
        Stmt::Item(_) => quote! { ::agdb::type_def::Expression::Block(&[]) },
        Stmt::Macro(m) => parse_stmt_macro(m, generics),
    }
}

fn parse_local(local: &syn::Local, generics: &[Generic]) -> TokenStream2 {
    let (name, ty) = parse_pattern(&local.pat, generics);
    let value = if let Some(init) = &local.init {
        let expr = parse_expression(&init.expr, generics);
        quote! { Some(&#expr) }
    } else {
        quote! { None }
    };

    quote! {
        ::agdb::type_def::Expression::Let {
            name: &#name,
            ty: #ty,
            value: #value,
        }
    }
}

fn parse_stmt_macro(m: &syn::StmtMacro, generics: &[Generic]) -> TokenStream2 {
    let name = path_to_string(&m.mac.path);
    parse_macro_by_name(&name, &m.mac.tokens, generics)
}

fn is_returnable(e: &Expr) -> bool {
    !matches!(
        e,
        Expr::Return(_)
            | Expr::Break(_)
            | Expr::Continue(_)
            | Expr::ForLoop(_)
            | Expr::While(_)
            | Expr::Loop(_)
            | Expr::Match(_)
            | Expr::If(_)
    )
}

// ---------------------------------------------------------------------------
// Array / Index
// ---------------------------------------------------------------------------

fn parse_array(e: &ExprArray, generics: &[Generic]) -> TokenStream2 {
    let elements = e.elems.iter().map(|elem| parse_expression(elem, generics));
    quote! {
        ::agdb::type_def::Expression::Array(&[#(#elements),*])
    }
}

fn parse_index(e: &ExprIndex, generics: &[Generic]) -> TokenStream2 {
    let base = parse_expression(&e.expr, generics);
    let index = parse_expression(&e.index, generics);
    quote! {
        ::agdb::type_def::Expression::Index {
            base: &#base,
            index: &#index,
        }
    }
}

// ---------------------------------------------------------------------------
// Assign
// ---------------------------------------------------------------------------

fn parse_assign(e: &syn::ExprAssign, generics: &[Generic]) -> TokenStream2 {
    let target = parse_expression(&e.left, generics);
    let value = parse_expression(&e.right, generics);
    quote! {
        ::agdb::type_def::Expression::Assign {
            target: &#target,
            value: &#value,
        }
    }
}

// ---------------------------------------------------------------------------
// Await
// ---------------------------------------------------------------------------

fn parse_await(e: &syn::ExprAwait, generics: &[Generic]) -> TokenStream2 {
    let expr = parse_expression(&e.base, generics);
    quote! {
        ::agdb::type_def::Expression::Await(&#expr)
    }
}

// ---------------------------------------------------------------------------
// Binary / Unary / Op
// ---------------------------------------------------------------------------

fn parse_binary(e: &ExprBinary, generics: &[Generic]) -> TokenStream2 {
    let left = parse_expression(&e.left, generics);
    let right = parse_expression(&e.right, generics);
    let op = parse_binop(&e.op);
    quote! {
        ::agdb::type_def::Expression::Binary {
            op: #op,
            left: &#left,
            right: &#right,
        }
    }
}

fn parse_unary(e: &ExprUnary, generics: &[Generic]) -> TokenStream2 {
    let expr = parse_expression(&e.expr, generics);
    let op = match &e.op {
        syn::UnOp::Deref(_) => quote! { ::agdb::type_def::Op::Deref },
        syn::UnOp::Not(_) => quote! { ::agdb::type_def::Op::Not },
        syn::UnOp::Neg(_) => quote! { ::agdb::type_def::Op::Neg },
        _ => crate::compile_error(e.op, format!("Unsupported unary operator: {:?}", e.op)),
    };
    quote! {
        ::agdb::type_def::Expression::Unary {
            op: #op,
            expr: &#expr,
        }
    }
}

fn parse_binop(op: &BinOp) -> TokenStream2 {
    match op {
        BinOp::Add(_) => quote! { ::agdb::type_def::Op::Add },
        BinOp::Sub(_) => quote! { ::agdb::type_def::Op::Sub },
        BinOp::Mul(_) => quote! { ::agdb::type_def::Op::Mul },
        BinOp::Div(_) => quote! { ::agdb::type_def::Op::Div },
        BinOp::Rem(_) => quote! { ::agdb::type_def::Op::Rem },
        BinOp::BitXor(_) => quote! { ::agdb::type_def::Op::BitXor },
        BinOp::BitAnd(_) => quote! { ::agdb::type_def::Op::BitAnd },
        BinOp::BitOr(_) => quote! { ::agdb::type_def::Op::BitOr },
        BinOp::Lt(_) => quote! { ::agdb::type_def::Op::Lt },
        BinOp::Gt(_) => quote! { ::agdb::type_def::Op::Gt },
        BinOp::And(_) => quote! { ::agdb::type_def::Op::And },
        BinOp::Or(_) => quote! { ::agdb::type_def::Op::Or },
        BinOp::Shl(_) => quote! { ::agdb::type_def::Op::Shl },
        BinOp::Shr(_) => quote! { ::agdb::type_def::Op::Shr },
        BinOp::Eq(_) => quote! { ::agdb::type_def::Op::Eq },
        BinOp::Le(_) => quote! { ::agdb::type_def::Op::Le },
        BinOp::Ne(_) => quote! { ::agdb::type_def::Op::Ne },
        BinOp::Ge(_) => quote! { ::agdb::type_def::Op::Ge },
        BinOp::AddAssign(_) => quote! { ::agdb::type_def::Op::AddAssign },
        BinOp::SubAssign(_) => quote! { ::agdb::type_def::Op::SubAssign },
        BinOp::MulAssign(_) => quote! { ::agdb::type_def::Op::MulAssign },
        BinOp::DivAssign(_) => quote! { ::agdb::type_def::Op::DivAssign },
        BinOp::RemAssign(_) => quote! { ::agdb::type_def::Op::RemAssign },
        BinOp::BitXorAssign(_) => quote! { ::agdb::type_def::Op::BitXorAssign },
        BinOp::BitAndAssign(_) => quote! { ::agdb::type_def::Op::BitAndAssign },
        BinOp::BitOrAssign(_) => quote! { ::agdb::type_def::Op::BitOrAssign },
        BinOp::ShlAssign(_) => quote! { ::agdb::type_def::Op::ShlAssign },
        BinOp::ShrAssign(_) => quote! { ::agdb::type_def::Op::ShrAssign },
        _ => crate::compile_error(op, "Unsupported binary operator"),
    }
}

// ---------------------------------------------------------------------------
// Call / MethodCall
// ---------------------------------------------------------------------------

fn parse_call(e: &ExprCall, generics: &[Generic]) -> TokenStream2 {
    let function = parse_expression(&e.func, generics);
    let args = e.args.iter().map(|arg| parse_expression(arg, generics));
    quote! {
        ::agdb::type_def::Expression::Call {
            recipient: None,
            function: &#function,
            args: &[#(#args),*],
        }
    }
}

fn parse_method_call(e: &ExprMethodCall, generics: &[Generic]) -> TokenStream2 {
    let recipient = parse_expression(&e.receiver, generics);
    let method = &e.method;
    let turbofish_generics = e
        .turbofish
        .as_ref()
        .map(|gt| {
            gt.args
                .iter()
                .filter_map(|ga| match ga {
                    GenericArgument::Type(ty) => {
                        Some(quote! { <#ty as ::agdb::type_def::TypeDefinition>::type_def })
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let args = e.args.iter().map(|arg| parse_expression(arg, generics));
    quote! {
        ::agdb::type_def::Expression::Call {
            recipient: Some(&#recipient),
            function: &::agdb::type_def::Expression::Path {
                ident: stringify!(#method),
                parent: None,
                generics: &[#(#turbofish_generics),*],
            },
            args: &[#(#args),*],
        }
    }
}

// ---------------------------------------------------------------------------
// Closure
// ---------------------------------------------------------------------------

fn parse_closure(e: &ExprClosure, generics: &[Generic]) -> TokenStream2 {
    let args: Vec<TokenStream2> = e
        .inputs
        .iter()
        .map(|pat| {
            let (name_tokens, ty_tokens) = parse_closure_arg(pat, generics);
            quote! {
                ::agdb::type_def::Variable {
                    name: #name_tokens,
                    ty: Some(#ty_tokens),
                }
            }
        })
        .collect();
    let async_fn = e.asyncness.is_some();
    let ret = parse_return_type(&e.output, generics);
    let body = match e.body.as_ref() {
        Expr::Block(body) => parse_block_stmts(&body.block, generics),
        other => {
            let expr = parse_expression(other, generics);
            vec![quote! { ::agdb::type_def::Expression::Return(Some(&#expr)) }]
        }
    };

    quote! {
        ::agdb::type_def::Expression::Closure(::agdb::type_def::Function {
            name: "",
            generics: &[],
            args: &[#(#args),*],
            ret: #ret,
            async_fn: #async_fn,
            body: &[#(#body),*],
        })
    }
}

fn parse_closure_arg(pat: &Pat, generics: &[Generic]) -> (TokenStream2, TokenStream2) {
    match pat {
        Pat::Type(p) => {
            let ty = generics_parser::parse_type(&p.ty, generics);
            match p.pat.as_ref() {
                Pat::Ident(pat_ident) => {
                    let name = pat_ident.ident.to_string();
                    (quote! { #name }, ty)
                }
                _ => (
                    crate::compile_error(
                        &p.pat,
                        format!(
                            "Expected identifier pattern, got: {}",
                            p.pat.to_token_stream()
                        ),
                    ),
                    ty,
                ),
            }
        }
        Pat::Ident(p) => {
            let name = p.ident.to_string();
            (
                quote! { #name },
                quote! { <() as ::agdb::type_def::TypeDefinition>::type_def },
            )
        }
        _ => (
            crate::compile_error(
                pat,
                format!(
                    "Unsupported closure argument pattern: {}",
                    pat.to_token_stream()
                ),
            ),
            quote! { <() as ::agdb::type_def::TypeDefinition>::type_def },
        ),
    }
}

fn parse_return_type(ret: &ReturnType, generics: &[Generic]) -> TokenStream2 {
    match ret {
        ReturnType::Default => quote! { <() as ::agdb::type_def::TypeDefinition>::type_def },
        ReturnType::Type(_, ty) => generics_parser::parse_type(ty, generics),
    }
}

// ---------------------------------------------------------------------------
// Field access / Tuple access
// ---------------------------------------------------------------------------

fn parse_field_access(e: &ExprField, generics: &[Generic]) -> TokenStream2 {
    let base = parse_expression(&e.base, generics);
    match &e.member {
        Member::Named(ident) => {
            quote! {
                ::agdb::type_def::Expression::FieldAccess {
                    base: &#base,
                    field: stringify!(#ident),
                }
            }
        }
        Member::Unnamed(index) => {
            let idx = index.index;
            quote! {
                ::agdb::type_def::Expression::TupleAccess {
                    base: &#base,
                    index: #idx,
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Loops
// ---------------------------------------------------------------------------

fn parse_for_loop(e: &syn::ExprForLoop, generics: &[Generic]) -> TokenStream2 {
    let (pattern, _) = parse_pattern(&e.pat, generics);
    let iterable = parse_expression(&e.expr, generics);
    let body = parse_block(&e.body, generics);
    quote! {
        ::agdb::type_def::Expression::For {
            pattern: &#pattern,
            iterable: &#iterable,
            body: &#body,
        }
    }
}

fn parse_loop(e: &syn::ExprLoop, generics: &[Generic]) -> TokenStream2 {
    let body = parse_block(&e.body, generics);
    quote! {
        ::agdb::type_def::Expression::While {
            condition: &::agdb::type_def::Expression::Literal(::agdb::type_def::LiteralValue::Bool(true)),
            body: &#body,
        }
    }
}

fn parse_while(e: &syn::ExprWhile, generics: &[Generic]) -> TokenStream2 {
    let condition = parse_expression(&e.cond, generics);
    let body = parse_block(&e.body, generics);
    quote! {
        ::agdb::type_def::Expression::While {
            condition: &#condition,
            body: &#body,
        }
    }
}

// ---------------------------------------------------------------------------
// If / Match
// ---------------------------------------------------------------------------

fn parse_if(e: &syn::ExprIf, generics: &[Generic]) -> TokenStream2 {
    let condition = parse_expression(&e.cond, generics);
    let then_branch = parse_block(&e.then_branch, generics);

    let else_branch = if let Some((_, else_expr)) = &e.else_branch {
        let else_tokens = match else_expr.as_ref() {
            Expr::If(else_if) => parse_if(else_if, generics),
            Expr::Block(else_block) => parse_block(&else_block.block, generics),
            _ => crate::compile_error(else_expr, "Unsupported else branch"),
        };
        quote! { Some(&#else_tokens) }
    } else {
        quote! { None }
    };

    quote! {
        ::agdb::type_def::Expression::If {
            condition: &#condition,
            then_branch: &#then_branch,
            else_branch: #else_branch,
        }
    }
}

fn parse_match(e: &syn::ExprMatch, generics: &[Generic]) -> TokenStream2 {
    let subject = parse_expression(&e.expr, generics);
    let mut branches = Vec::new();
    let mut else_branch: Option<TokenStream2> = None;

    for arm in &e.arms {
        if matches!(&arm.pat, Pat::Wild(_)) {
            else_branch = Some(parse_match_arm_body(&arm.body, generics));
        } else {
            let condition = parse_match_condition(&subject, &arm.pat, generics);
            let condition_with_guard = if let Some((_, guard)) = &arm.guard {
                let guard_expr = parse_expression(guard, generics);
                quote! {
                    ::agdb::type_def::Expression::Binary {
                        op: ::agdb::type_def::Op::And,
                        left: &#condition,
                        right: &#guard_expr,
                    }
                }
            } else {
                condition
            };
            let body = parse_match_arm_body(&arm.body, generics);
            branches.push((condition_with_guard, body));
        }
    }

    if branches.is_empty() {
        else_branch.expect("Match expression must have at least one arm")
    } else {
        branches
            .iter()
            .rev()
            .fold(else_branch, |else_br, (cond, body)| {
                let else_part = if let Some(eb) = else_br {
                    eb
                } else {
                    quote! { ::agdb::type_def::Expression::Block(&[]) }
                };
                Some(quote! {
                    ::agdb::type_def::Expression::If {
                        condition: &#cond,
                        then_branch: &#body,
                        else_branch: Some(&#else_part),
                    }
                })
            })
            .expect("At least one match arm present")
    }
}

fn parse_match_arm_body(body: &Expr, generics: &[Generic]) -> TokenStream2 {
    match body {
        Expr::Block(b) => parse_block(&b.block, generics),
        Expr::Tuple(t) if t.elems.is_empty() => {
            quote! { ::agdb::type_def::Expression::Block(&[]) }
        }
        expr => {
            let inner = parse_expression(expr, generics);
            quote! { ::agdb::type_def::Expression::Block(&[#inner]) }
        }
    }
}

fn parse_match_condition(subject: &TokenStream2, pat: &Pat, generics: &[Generic]) -> TokenStream2 {
    if let Pat::Or(p) = pat {
        return parse_match_or(subject, p, generics);
    }

    let (rhs, _) = parse_pattern(pat, generics);
    quote! {
        ::agdb::type_def::Expression::Binary {
            op: ::agdb::type_def::Op::Eq,
            left: &#subject,
            right: &#rhs,
        }
    }
}

fn parse_match_or(subject: &TokenStream2, pat_or: &PatOr, generics: &[Generic]) -> TokenStream2 {
    let conds: Vec<TokenStream2> = pat_or
        .cases
        .iter()
        .map(|subpat| parse_match_condition(subject, subpat, generics))
        .collect();
    let mut iter = conds.into_iter();
    let first = iter.next().expect("Or pattern must have cases");
    iter.fold(first, |acc, next| {
        quote! {
            ::agdb::type_def::Expression::Binary {
                op: ::agdb::type_def::Op::Or,
                left: &#acc,
                right: &#next,
            }
        }
    })
}

// ---------------------------------------------------------------------------
// Let (expression form, e.g. `if let`)
// ---------------------------------------------------------------------------

fn parse_let_expr(e: &syn::ExprLet, generics: &[Generic]) -> TokenStream2 {
    let (name, ty) = parse_pattern(&e.pat, generics);
    let value = parse_expression(&e.expr, generics);
    quote! {
        ::agdb::type_def::Expression::Let {
            name: &#name,
            ty: #ty,
            value: Some(&#value),
        }
    }
}

// ---------------------------------------------------------------------------
// Literal
// ---------------------------------------------------------------------------

fn parse_literal(lit: &Lit) -> TokenStream2 {
    match lit {
        Lit::Str(s) => {
            let value = s.value();
            quote! {
                ::agdb::type_def::Expression::Literal(::agdb::type_def::LiteralValue::Str(#value))
            }
        }
        Lit::Int(i) => {
            let suffix = i.suffix();
            match suffix {
                "i8" => {
                    let v = i.base10_parse::<i8>().unwrap();
                    quote! { ::agdb::type_def::Expression::Literal(::agdb::type_def::LiteralValue::I8(#v)) }
                }
                "i16" => {
                    let v = i.base10_parse::<i16>().unwrap();
                    quote! { ::agdb::type_def::Expression::Literal(::agdb::type_def::LiteralValue::I16(#v)) }
                }
                "i32" | "" => {
                    let v = i.base10_parse::<i32>().unwrap();
                    quote! { ::agdb::type_def::Expression::Literal(::agdb::type_def::LiteralValue::I32(#v)) }
                }
                "u8" => {
                    let v = i.base10_parse::<u8>().unwrap();
                    quote! { ::agdb::type_def::Expression::Literal(::agdb::type_def::LiteralValue::U8(#v)) }
                }
                "u16" => {
                    let v = i.base10_parse::<u16>().unwrap();
                    quote! { ::agdb::type_def::Expression::Literal(::agdb::type_def::LiteralValue::U16(#v)) }
                }
                "u32" => {
                    let v = i.base10_parse::<u32>().unwrap();
                    quote! { ::agdb::type_def::Expression::Literal(::agdb::type_def::LiteralValue::U32(#v)) }
                }
                "u64" => {
                    let v = i.base10_parse::<u64>().unwrap();
                    quote! { ::agdb::type_def::Expression::Literal(::agdb::type_def::LiteralValue::U64(#v)) }
                }
                "usize" => {
                    let v = i.base10_parse::<usize>().unwrap();
                    quote! { ::agdb::type_def::Expression::Literal(::agdb::type_def::LiteralValue::Usize(#v)) }
                }
                _ => crate::compile_error(i, format!("Unsupported integer suffix: {suffix}")),
            }
        }
        Lit::Float(f) => {
            let suffix = f.suffix();
            match suffix {
                "f32" => {
                    let v = f.base10_parse::<f32>().unwrap();
                    quote! { ::agdb::type_def::Expression::Literal(::agdb::type_def::LiteralValue::F32(#v)) }
                }
                "f64" | "" => {
                    let v = f.base10_parse::<f64>().unwrap();
                    quote! { ::agdb::type_def::Expression::Literal(::agdb::type_def::LiteralValue::F64(#v)) }
                }
                _ => crate::compile_error(f, format!("Unsupported float suffix: {suffix}")),
            }
        }
        Lit::Bool(b) => {
            let value = b.value;
            quote! {
                ::agdb::type_def::Expression::Literal(::agdb::type_def::LiteralValue::Bool(#value))
            }
        }
        Lit::Char(c) => {
            let value = c.value().to_string();
            quote! {
                ::agdb::type_def::Expression::Literal(::agdb::type_def::LiteralValue::Str(#value))
            }
        }
        _ => crate::compile_error(lit, format!("Unsupported literal: {:?}", lit)),
    }
}

// ---------------------------------------------------------------------------
// Macro (format!, vec!, etc.)
// ---------------------------------------------------------------------------

fn parse_macro(e: &ExprMacro, generics: &[Generic]) -> TokenStream2 {
    let name = path_to_string(&e.mac.path);
    parse_macro_by_name(&name, &e.mac.tokens, generics)
}

fn parse_macro_by_name(
    name: &str,
    tokens: &proc_macro2::TokenStream,
    generics: &[Generic],
) -> TokenStream2 {
    let args: syn::punctuated::Punctuated<Expr, syn::token::Comma> = syn::parse::Parser::parse2(
        syn::punctuated::Punctuated::parse_terminated,
        tokens.clone(),
    )
    .unwrap_or_default();

    match name {
        "vec" => {
            let elements = args.iter().map(|arg| parse_expression(arg, generics));
            quote! {
                ::agdb::type_def::Expression::Array(&[#(#elements),*])
            }
        }
        "format" => {
            let mut args_iter = args.into_iter();
            let format_string_expr = args_iter
                .next()
                .expect("format! requires at least one argument");
            let format_string = match extract_format_string(&format_string_expr) {
                Ok(v) => v,
                Err(err) => return err,
            };
            let (fmt_str, fmt_args) =
                extract_format_parts(&format_string, &mut args_iter, generics);
            quote! {
                ::agdb::type_def::Expression::Format {
                    format_string: #fmt_str,
                    args: &[#(#fmt_args),*],
                }
            }
        }
        // Common macros treated as function calls
        "panic" | "todo" | "unimplemented" | "println" | "eprintln" | "dbg" | "assert"
        | "assert_eq" | "assert_ne" | "debug_assert" | "debug_assert_eq" | "debug_assert_ne"
        | "matches" | "unreachable" | "write" | "writeln" | "bail" => {
            let macro_args = args.iter().map(|arg| parse_expression(arg, generics));
            quote! {
                ::agdb::type_def::Expression::Call {
                    recipient: None,
                    function: &::agdb::type_def::Expression::Path {
                        ident: #name,
                        parent: None,
                        generics: &[],
                    },
                    args: &[#(#macro_args),*],
                }
            }
        }
        _ => crate::compile_error(tokens, format!("Unsupported macro: {name}")),
    }
}

fn extract_format_string(e: &Expr) -> Result<String, TokenStream2> {
    match e {
        Expr::Lit(expr_lit) => {
            if let Lit::Str(lit_str) = &expr_lit.lit {
                Ok(lit_str.value())
            } else {
                Err(crate::compile_error(
                    expr_lit,
                    "First argument to format! must be a string literal",
                ))
            }
        }
        _ => Err(crate::compile_error(
            e,
            "First argument to format! must be a string literal",
        )),
    }
}

fn extract_format_parts(
    format_string: &str,
    args_iter: &mut impl Iterator<Item = Expr>,
    generics: &[Generic],
) -> (TokenStream2, Vec<TokenStream2>) {
    let mut args = Vec::new();
    let mut fmt_str = String::new();
    let mut chars = format_string.chars();

    while let Some(c) = chars.next() {
        fmt_str.push(c);

        if c == '{'
            && let Some(next) = chars.next()
        {
            if next == '{' {
                fmt_str.push(next);
                continue; // escaped brace
            }

            fmt_str.push('}');

            if next == '}' {
                // positional argument
                let arg = args_iter
                    .next()
                    .expect("not enough arguments for format string");
                args.push(parse_expression(&arg, generics));
            } else {
                // named argument
                let mut ident = next.to_string();
                for nc in chars.by_ref() {
                    if nc == '}' {
                        break;
                    }
                    ident.push(nc);
                }
                args.push(quote! { ::agdb::type_def::Expression::Ident(#ident) });
            }
        }
    }

    (quote! { #fmt_str }, args)
}

// ---------------------------------------------------------------------------
// Path
// ---------------------------------------------------------------------------

fn parse_path(path: &Path) -> TokenStream2 {
    let mut iter = path.segments.iter();
    let first = iter.next().expect("path should have at least one segment");

    // Single-segment path with no generics => Ident
    if path.segments.len() == 1 && matches!(first.arguments, PathArguments::None) {
        let ident = &first.ident;
        return quote! {
            ::agdb::type_def::Expression::Ident(stringify!(#ident))
        };
    }

    let first_segment = parse_path_segment(first, quote! { None });
    iter.fold(first_segment, |parent, segment| {
        parse_path_segment(segment, quote! { Some(&#parent) })
    })
}

fn parse_path_segment(segment: &PathSegment, parent: TokenStream2) -> TokenStream2 {
    let ident = &segment.ident;
    let generics = match &segment.arguments {
        PathArguments::AngleBracketed(args) => args
            .args
            .iter()
            .filter_map(|ga| match ga {
                GenericArgument::Type(ty) => {
                    Some(quote! { <#ty as ::agdb::type_def::TypeDefinition>::type_def })
                }
                _ => None,
            })
            .collect::<Vec<_>>(),
        PathArguments::Parenthesized(args) => args
            .inputs
            .iter()
            .map(|ty| quote! { <#ty as ::agdb::type_def::TypeDefinition>::type_def })
            .collect::<Vec<_>>(),
        PathArguments::None => Vec::new(),
    };

    quote! {
        ::agdb::type_def::Expression::Path {
            ident: stringify!(#ident),
            parent: #parent,
            generics: &[#(#generics),*],
        }
    }
}

fn path_to_string(path: &Path) -> String {
    path.segments
        .last()
        .expect("path should not be empty")
        .ident
        .to_string()
}

// ---------------------------------------------------------------------------
// Range
// ---------------------------------------------------------------------------

fn parse_range(e: &ExprRange, generics: &[Generic]) -> TokenStream2 {
    let start = if let Some(start) = &e.start {
        let s = parse_expression(start, generics);
        quote! { Some(&#s) }
    } else {
        quote! { None }
    };
    let end = if let Some(end) = &e.end {
        let v = parse_expression(end, generics);
        quote! { Some(&#v) }
    } else {
        quote! { None }
    };
    let inclusive = matches!(e.limits, syn::RangeLimits::Closed(_));
    quote! {
        ::agdb::type_def::Expression::Range {
            start: #start,
            end: #end,
            inclusive: #inclusive,
        }
    }
}

// ---------------------------------------------------------------------------
// Reference / Return / Try
// ---------------------------------------------------------------------------

fn parse_reference(e: &ExprReference, generics: &[Generic]) -> TokenStream2 {
    let expr = parse_expression(&e.expr, generics);
    quote! {
        ::agdb::type_def::Expression::Reference(&#expr)
    }
}

fn parse_return(e: &syn::ExprReturn, generics: &[Generic]) -> TokenStream2 {
    if let Some(expr) = &e.expr {
        let parsed = parse_expression(expr, generics);
        quote! {
            ::agdb::type_def::Expression::Return(Some(&#parsed))
        }
    } else {
        quote! {
            ::agdb::type_def::Expression::Return(None)
        }
    }
}

fn parse_try(e: &ExprTry, generics: &[Generic]) -> TokenStream2 {
    let expr = parse_expression(&e.expr, generics);
    quote! {
        ::agdb::type_def::Expression::Try(&#expr)
    }
}

// ---------------------------------------------------------------------------
// Struct / Tuple
// ---------------------------------------------------------------------------

fn parse_struct(e: &ExprStruct, generics: &[Generic]) -> TokenStream2 {
    let path = parse_path(&e.path);
    let fields = e.fields.iter().map(|f| parse_struct_field(f, generics));
    quote! {
        ::agdb::type_def::Expression::Struct {
            name: &#path,
            fields: &[#(#fields),*],
        }
    }
}

fn parse_struct_field(field: &FieldValue, generics: &[Generic]) -> TokenStream2 {
    let field_name = match &field.member {
        Member::Named(ident) => ident,
        Member::Unnamed(_) => {
            return crate::compile_error(
                &field.member,
                "Unnamed fields are not supported in struct expressions",
            );
        }
    };
    let field_value = parse_expression(&field.expr, generics);
    quote! {
        (stringify!(#field_name), #field_value)
    }
}

fn parse_tuple(e: &syn::ExprTuple, generics: &[Generic]) -> TokenStream2 {
    let elements = e.elems.iter().map(|elem| parse_expression(elem, generics));
    quote! {
        ::agdb::type_def::Expression::Tuple(&[#(#elements),*])
    }
}

// ---------------------------------------------------------------------------
// Patterns (used in let, for, match, closure, etc.)
// ---------------------------------------------------------------------------

/// Returns `(name_tokens, type_tokens)` where type_tokens is `None` or
/// `Some(fn_ptr)`.
fn parse_pattern(pat: &Pat, generics: &[Generic]) -> (TokenStream2, TokenStream2) {
    match pat {
        Pat::Ident(p) => {
            let name = &p.ident;
            (
                quote! { ::agdb::type_def::Expression::Ident(stringify!(#name)) },
                quote! { None },
            )
        }
        Pat::Lit(p) => {
            let lit = parse_literal(&p.lit);
            (lit, quote! { None })
        }
        Pat::Type(p) => {
            let (name, _) = parse_pattern(&p.pat, generics);
            let ty = generics_parser::parse_type(&p.ty, generics);
            (name, quote! { Some(#ty) })
        }
        Pat::Or(p) => {
            let conds: Vec<TokenStream2> = p
                .cases
                .iter()
                .map(|subpat| parse_pattern(subpat, generics).0)
                .collect();
            let mut iter = conds.into_iter();
            let first = iter.next().expect("Or pattern must have cases");
            (
                iter.fold(first, |acc, next| {
                    quote! {
                        ::agdb::type_def::Expression::Binary {
                            op: ::agdb::type_def::Op::Or,
                            left: &#acc,
                            right: &#next,
                        }
                    }
                }),
                quote! { None },
            )
        }
        Pat::Paren(p) => parse_pattern(&p.pat, generics),
        Pat::Path(p) => {
            let path = parse_path(&p.path);
            (path, quote! { None })
        }
        Pat::Reference(p) => parse_pattern(&p.pat, generics),
        Pat::Slice(p) => {
            let elems = p.elems.iter().map(|elem| parse_pattern(elem, generics).0);
            (
                quote! { ::agdb::type_def::Expression::Array(&[#(#elems),*]) },
                quote! { None },
            )
        }
        Pat::Struct(p) => {
            let path = parse_path(&p.path);
            let fields = p.fields.iter().map(|f| parse_pattern(&f.pat, generics).0);
            (
                quote! {
                    ::agdb::type_def::Expression::StructPattern {
                        name: &#path,
                        fields: &[#(#fields),*],
                    }
                },
                quote! { None },
            )
        }
        Pat::Tuple(p) => {
            let elems = p.elems.iter().map(|elem| parse_pattern(elem, generics).0);
            (
                quote! { ::agdb::type_def::Expression::Tuple(&[#(#elems),*]) },
                quote! { None },
            )
        }
        Pat::TupleStruct(p) => {
            let path = parse_path(&p.path);
            let elems = p.elems.iter().map(|elem| parse_pattern(elem, generics).0);
            (
                quote! {
                    ::agdb::type_def::Expression::TupleStruct {
                        name: &#path,
                        expressions: &[#(#elems),*],
                    }
                },
                quote! { None },
            )
        }
        Pat::Wild(_) => (
            quote! { ::agdb::type_def::Expression::Wild },
            quote! { None },
        ),
        _ => (
            crate::compile_error(
                pat,
                format!("Unsupported pattern: {}", pat.to_token_stream()),
            ),
            quote! { None },
        ),
    }
}
