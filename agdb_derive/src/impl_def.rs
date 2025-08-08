use proc_macro::TokenStream;
use quote::ToTokens;
use quote::quote;
use syn::Expr;
use syn::Generics;
use syn::ImplItem;
use syn::ItemImpl;
use syn::PathArguments;
use syn::ReturnType;
use syn::TypeParamBound;
use syn::punctuated::Punctuated;
use syn::token::Comma;

pub fn impl_def(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut ret = item.clone();

    let impl_block = syn::parse_macro_input!(item as ItemImpl);

    let ty = impl_block.self_ty;
    let ty_g = impl_block.generics;
    let mut funcs = vec![];

    for i in impl_block.items {
        if let ImplItem::Fn(f) = i {
            funcs.push(parse_function(f));
        }
    }

    let t = quote! {
        impl #ty_g ::agdb::api::ApiFunctions for #ty {
            fn functions() -> Vec<::agdb::api::Function> {
                vec![#(#funcs),*]
            }
         }
    };

    ret.extend(Into::<TokenStream>::into(t));
    ret
}

fn parse_function(f: syn::ImplItemFn) -> proc_macro2::TokenStream {
    let name = f.sig.ident.to_string();
    let ret_ty = return_type(&f);
    let mut args = vec![];
    let mut exprs: Vec<proc_macro2::TokenStream> = vec![];

    for a in f.sig.inputs {
        if let syn::FnArg::Typed(t) = a {
            args.push(parse_arg(t, &f.sig.generics));
        }
    }

    for stmt in f.block.stmts {
        exprs.push(parse_stmt(&stmt));
    }

    let api_func = quote! {
        ::agdb::api::Function {
            name: #name,
            args: vec![#(#args),*],
            ret: #ret_ty,
            expressions: vec![#(#exprs),*],
        }
    };
    api_func
}

fn parse_arg(t: syn::PatType, generics: &Generics) -> proc_macro2::TokenStream {
    let name = t.pat.to_token_stream().to_string();
    let ty = arg_type(&t.ty, generics);
    quote! { ::agdb::api::NamedType { name: #name, ty: #ty } }
}

fn arg_type(t: &syn::Type, generics: &Generics) -> proc_macro2::TokenStream {
    let ty = extract_type(t);
    let t_str = ty.to_token_stream().to_string();

    generics
        .type_params()
        .find_map(|g| {
            if g.ident == t_str {
                if let Some(TypeParamBound::Trait(bound)) = g.bounds.first() {
                    if let Some(bound) = bound.path.segments.first() {
                        let bound_str = bound.ident.to_string();
                        if bound_str == "Into" {
                            if let PathArguments::AngleBracketed(ty) = &bound.arguments {
                                if let syn::GenericArgument::Type(ty) = &ty.args[0] {
                                    return Some(
                                        quote! { <::agdb::#ty as ::agdb::api::ApiDefinition>::def },
                                    );
                                }
                            }
                        } else if bound_str == "DbUserValue" {
                            return db_user_value(t);
                        }
                    }
                }
            }

            None
        })
        .unwrap_or(quote! { <#t as ::agdb::api::ApiDefinition>::def })
}

fn extract_type(t: &syn::Type) -> &syn::Type {
    match t {
        syn::Type::Array(type_array) => &type_array.elem,
        syn::Type::Reference(type_reference) => extract_type(&type_reference.elem),
        syn::Type::Slice(type_slice) => &type_slice.elem,
        _ => t,
    }
}

fn db_user_value(t: &syn::Type) -> Option<proc_macro2::TokenStream> {
    match t {
        syn::Type::Path(_) => Some(quote! { || ::agdb::api::Type::User }),
        syn::Type::Array(_) => Some(
            quote! { || ::agdb::api::Type::List(::agdb::api::List { name: format!("List_{}", ::agdb::api::Type::User.name()), ty: || ::agdb::api::Type::User }) },
        ),
        syn::Type::Reference(reference) => db_user_value(&reference.elem),
        syn::Type::Slice(_) => Some(
            quote! { || ::agdb::api::Type::List(::agdb::api::List { name: format!("List_{}", ::agdb::api::Type::User.name()), ty: || ::agdb::api::Type::User }) },
        ),
        _ => None,
    }
}

fn return_type(f: &syn::ImplItemFn) -> proc_macro2::TokenStream {
    if let ReturnType::Type(_, t) = &f.sig.output {
        quote! { Some(<#t as ::agdb::api::ApiDefinition>::def) }
    } else {
        quote! { None }
    }
}

fn parse_stmt(stmt: &syn::Stmt) -> proc_macro2::TokenStream {
    match stmt {
        syn::Stmt::Local(local) => parse_local_stmt(local),
        syn::Stmt::Expr(expr, _semi) => {
            let expr_tokens = parse_expr(expr);
            quote! { #expr_tokens }
        }
        syn::Stmt::Item(_item) => unimplemented!("Item statements are not supported"),
        syn::Stmt::Macro(_macro) => unimplemented!("Macro statements are not supported"),
    }
}

fn parse_local_stmt(local: &syn::Local) -> proc_macro2::TokenStream {
    let name = match &local.pat {
        syn::Pat::Ident(ident) => ident.ident.to_string(),
        syn::Pat::Lit(lit) => parse_literal_ident(lit),
        syn::Pat::Tuple(t) => {
            let names: Vec<String> = t
                .elems
                .iter()
                .map(|e| e.to_token_stream().to_string())
                .collect();
            names.join(", ")
        }
        _ => unimplemented!(
            "Only identifiers, literals and tuples are supported for let statements: {:?}",
            local.pat
        ),
    };
    let value = match &local.init {
        Some(init) => {
            let expr_tokens = parse_expr(&init.expr);
            quote! { Some(Box::new(#expr_tokens)) }
        }
        None => quote! { None },
    };
    quote! {
        ::agdb::api::Expression::Let {
            name: #name,
            ty: None,
            value: #value,
        }
    }
}

fn parse_expr(expr: &syn::Expr) -> proc_macro2::TokenStream {
    match expr {
        syn::Expr::Lit(e) => parse_literal(e),
        syn::Expr::Path(e) => {
            let ident = e
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();
            quote! { ::agdb::api::Expression::Variable(#ident) }
        }
        syn::Expr::Call(e) => {
            let (recipient, func) = match &*e.func {
                syn::Expr::Path(path) => {
                    let mut segments: Vec<String> = path
                        .path
                        .segments
                        .iter()
                        .map(|s| s.ident.to_string())
                        .collect();
                    let func = segments.pop().unwrap_or_default();
                    let recipient = if let Some(last) = segments.pop() {
                        quote! { Some(Box::new(::agdb::api::Expression::Variable(#last))) }
                    } else {
                        quote! { None }
                    };
                    (recipient, func)
                }
                _ => panic!("Expected a path for function call"),
            };
            let args = e.args.iter().map(parse_expr);
            quote! {
                ::agdb::api::Expression::Call {
                    recipient: #recipient,
                    function: #func,
                    args: vec![#(#args),*],
                }
            }
        }
        syn::Expr::Assign(e) => {
            let left = parse_expr(&e.left);
            let right = parse_expr(&e.right);
            quote! {
                ::agdb::api::Expression::Assign {
                    target: Box::new(#left),
                    value: Box::new(#right),
                }
            }
        }
        syn::Expr::If(e) => {
            let cond = parse_expr(&e.cond);
            let then_branch = {
                let stmts = &e.then_branch.stmts;
                let exprs = stmts.iter().map(parse_stmt);
                quote! { Box::new(::agdb::api::Expression::Block(vec![#(#exprs),*])) }
            };
            let else_branch = if let Some((_, else_expr)) = &e.else_branch {
                let else_tokens = parse_expr(else_expr);
                quote! { Some(Box::new(#else_tokens)) }
            } else {
                quote! { None }
            };
            quote! {
                ::agdb::api::Expression::If {
                    condition: Box::new(#cond),
                    then_branch: #then_branch,
                    else_branch: #else_branch,
                }
            }
        }
        syn::Expr::Block(e) => {
            let exprs = e.block.stmts.iter().map(parse_stmt);
            quote! {
                ::agdb::api::Expression::Block(vec![#(#exprs),*])
            }
        }
        syn::Expr::Return(e) => {
            if let Some(expr) = &e.expr {
                let val = parse_expr(expr);
                quote! { ::agdb::api::Expression::Return(Some(Box::new(#val))) }
            } else {
                quote! { ::agdb::api::Expression::Return(None) }
            }
        }
        syn::Expr::Field(e) => {
            let base = parse_expr(&e.base);
            let field = e.member.to_token_stream().to_string();
            quote! {
                ::agdb::api::Expression::FieldAccess {
                    base: Box::new(#base),
                    field: #field,
                }
            }
        }
        syn::Expr::Index(e) => {
            let base = parse_expr(&e.expr);
            let index = parse_expr(&e.index);
            quote! {
                ::agdb::api::Expression::Index {
                    base: Box::new(#base),
                    index: Box::new(#index),
                }
            }
        }
        syn::Expr::While(e) => {
            let cond = parse_expr(&e.cond);
            let body = {
                let stmts = &e.body.stmts;
                let exprs = stmts.iter().map(parse_stmt);
                quote! { Box::new(::agdb::api::Expression::Block(vec![#(#exprs),*])) }
            };
            quote! {
                ::agdb::api::Expression::While {
                    condition: Box::new(#cond),
                    body: #body,
                }
            }
        }

        syn::Expr::MethodCall(e) => {
            let func = &e.method.to_string();
            let args = e.args.iter().map(parse_expr);
            let recipient = parse_expr(&e.receiver);
            quote! {
                ::agdb::api::Expression::Call {
                    recipient: Some(Box::new(#recipient)),
                    function: #func,
                    args: vec![#(#args),*],
                }
            }
        }
        syn::Expr::Struct(e) => {
            let path = e.path.to_token_stream().to_string();
            let fields = e.fields.iter().map(|f| {
                let name = match &f.member {
                    syn::Member::Named(ident) => ident.to_string(),
                    syn::Member::Unnamed(index) => index.index.to_string(),
                };
                let value = parse_expr(&f.expr);
                quote! { (#name, Box::new(#value)) }
            });
            quote! {
                ::agdb::api::Expression::Struct {
                    name: #path,
                    fields: vec![#(#fields),*],
                }
            }
        }
        syn::Expr::Reference(e) => parse_expr(&e.expr),
        syn::Expr::Macro(e) => {
            let macro_name = e
                .mac
                .path
                .segments
                .last()
                .expect("expected macro name")
                .ident
                .to_string();

            if macro_name == "vec" {
                let args: Punctuated<Expr, Comma> = syn::parse2(e.mac.tokens.clone())
                    .map(|args: syn::ExprArray| args.elems)
                    .unwrap_or_default();
                let args = args.iter().map(parse_expr);

                quote! {
                    ::agdb::api::Expression::Array {
                        elements: vec![#(#args),*],
                    }
                }
            } else if macro_name == "format" {
                let args: Punctuated<Expr, Comma> = syn::parse2(e.mac.tokens.clone())
                    .map(|args: syn::ExprArray| args.elems)
                    .unwrap_or_default();
                let args = args.iter().map(parse_expr);

                quote! {
                    ::agdb::api::Expression::Call {
                        recipient: None,
                        function: "format",
                        args: vec![#(#args),*],
                    }
                }
            } else {
                unimplemented!("Unsupported macro: {e:?}");
            }
        }
        syn::Expr::Let(e) => {
            let pat = e.pat.to_token_stream().to_string();
            let expr = parse_expr(&e.expr);
            quote! {
                ::agdb::api::Expression::Let {
                    name: #pat,
                    ty: None,
                    value: Some(Box::new(#expr)),
                }
            }
        }
        syn::Expr::Binary(e) => {
            let op = match &e.op {
                syn::BinOp::Add(_) => quote! { ::agdb::api::Op::Add },
                syn::BinOp::Sub(_) => quote! { ::agdb::api::Op::Sub },
                syn::BinOp::Mul(_) => quote! { ::agdb::api::Op::Mul },
                syn::BinOp::Div(_) => quote! { ::agdb::api::Op::Div },
                syn::BinOp::Rem(_) => quote! { ::agdb::api::Op::Rem },
                syn::BinOp::And(_) => quote! { ::agdb::api::Op::And },
                syn::BinOp::Or(_) => quote! { ::agdb::api::Op::Or },
                syn::BinOp::BitXor(_) => quote! { ::agdb::api::Op::BitXor },
                syn::BinOp::BitAnd(_) => quote! { ::agdb::api::Op::BitAnd },
                syn::BinOp::BitOr(_) => quote! { ::agdb::api::Op::BitOr },
                syn::BinOp::Shl(_) => quote! { ::agdb::api::Op::Shl },
                syn::BinOp::Shr(_) => quote! { ::agdb::api::Op::Shr },
                syn::BinOp::Eq(_) => quote! { ::agdb::api::Op::Eq },
                syn::BinOp::Lt(_) => quote! { ::agdb::api::Op::Lt },
                syn::BinOp::Le(_) => quote! { ::agdb::api::Op::Le },
                syn::BinOp::Ne(_) => quote! { ::agdb::api::Op::Ne },
                syn::BinOp::Ge(_) => quote! { ::agdb::api::Op::Ge },
                syn::BinOp::Gt(_) => quote! { ::agdb::api::Op::Gt },
                _ => unimplemented!("Unsupported binary operator: {e:?}"),
            };
            let left = parse_expr(&e.left);
            let right = parse_expr(&e.right);
            quote! {
                ::agdb::api::Expression::Binary {
                    op: #op,
                    left: Box::new(#left),
                    right: Box::new(#right),
                }
            }
        }
        syn::Expr::Unary(e) => {
            let op = match &e.op {
                syn::UnOp::Deref(_) => quote! { ::agdb::api::Op::Not },
                syn::UnOp::Not(_) => quote! { ::agdb::api::Op::Not },
                syn::UnOp::Neg(_) => quote! { ::agdb::api::Op::Neg },
                _ => unimplemented!("Unsupported unary operator: {e:?}"),
            };
            let expr = parse_expr(&e.expr);
            quote! {
                ::agdb::api::Expression::Unary {
                    op: #op,
                    expr: Box::new(#expr),
                }
            }
        }
        syn::Expr::Closure(e) => {
            let body = match &*e.body {
                syn::Expr::Block(block) => {
                    let exprs = block.block.stmts.iter().map(parse_stmt);
                    quote! { vec![#(#exprs),*] }
                }
                expr => {
                    let parsed_expr = parse_expr(expr);
                    quote! { vec![#parsed_expr] }
                }
            };

            let ret_type = if let syn::ReturnType::Type(_, ty) = &e.output {
                quote! { Some(<#ty as ::agdb::api::ApiDefinition>::def) }
            } else {
                quote! { None }
            };

            quote! {
                ::agdb::api::Expression::Closure {
                    ret: #ret_type,
                    body: #body,
                }
            }
        }
        syn::Expr::Try(e) => {
            // skip the ? operator
            let expr = parse_expr(&e.expr);
            quote! {
                #expr
            }
        }
        syn::Expr::Await(e) => {
            // skip the await operator
            let expr = parse_expr(&e.base);
            quote! {
                #expr
            }
        }
        _ => {
            unimplemented!("Unsupported expression type: {expr:?}");
        }
    }
}

fn parse_literal_ident(e: &syn::PatLit) -> String {
    match &e.lit {
        syn::Lit::Int(v) => v.to_string(),
        _ => unimplemented!("Unsupported literal identifier type, only Int is supported"),
    }
}

fn parse_literal(e: &syn::PatLit) -> proc_macro2::TokenStream {
    match &e.lit {
        syn::Lit::Int(v) => {
            let v_str = v.to_string();
            quote! { ::agdb::api::Expression::Literal(::agdb::api::LiteralValue::I64(#v_str)) }
        }
        syn::Lit::Float(v) => {
            let v_str = v.to_string();
            quote! { ::agdb::api::Expression::Literal(::agdb::api::LiteralValue::F64(#v_str)) }
        }
        syn::Lit::Str(v) => {
            quote! { ::agdb::api::Expression::Literal(::agdb::api::LiteralValue::String(#v)) }
        }
        syn::Lit::Bool(v) => {
            quote! { ::agdb::api::Expression::Literal(::agdb::api::LiteralValue::Bool(#v)) }
        }
        _ => unimplemented!("Unsupported literal type"),
    }
}
