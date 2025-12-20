use super::expression;
use super::expression::pattern;
use proc_macro2::TokenStream;
use syn::Stmt;
use syn::token::Semi;

#[derive(Clone, Copy)]
pub(crate) struct ExpressionContext<'a> {
    pub fn_name: &'a str,
    pub generics: &'a [String],
    pub level: usize,
    pub semicolon: bool,
    pub last: bool,
}

impl ExpressionContext<'_> {
    pub(crate) fn new<'a>(fn_name: &'a str, generics: &'a [String]) -> ExpressionContext<'a> {
        ExpressionContext {
            fn_name,
            generics,
            level: 0,
            semicolon: false,
            last: false,
        }
    }

    pub(crate) fn inner(&self) -> ExpressionContext<'_> {
        ExpressionContext {
            fn_name: self.fn_name,
            generics: self.generics,
            level: self.level + 1,
            semicolon: false,
            last: false,
        }
    }

    pub(crate) fn last(&self) -> ExpressionContext<'_> {
        ExpressionContext {
            fn_name: self.fn_name,
            generics: self.generics,
            level: self.level,
            semicolon: self.semicolon,
            last: true,
        }
    }

    pub(crate) fn semicolon(&self, semi: &Option<Semi>) -> ExpressionContext<'_> {
        ExpressionContext {
            fn_name: self.fn_name,
            generics: self.generics,
            level: self.level,
            semicolon: semi.is_some(),
            last: self.last,
        }
    }
}

pub(crate) fn parse_statements(stmts: &[Stmt], context: ExpressionContext) -> Vec<TokenStream> {
    stmts
        .iter()
        .map(|stmt| parse_statement(stmt, context))
        .collect()
}

pub(crate) fn parse_statement(stmt: &Stmt, context: ExpressionContext) -> TokenStream {
    match stmt {
        Stmt::Local(local) => pattern::parse_let(
            &local.pat,
            local.init.as_ref().map(|i| i.expr.as_ref()),
            context,
        ),
        Stmt::Item(item) => panic!("nested items not supported: {item:?}"),
        Stmt::Expr(expr, semi) => expression::parse_expression(expr, context.semicolon(semi)),
        Stmt::Macro(stmt_macro) => panic!("stmt_macro not supported: {stmt_macro:?}"),
    }
}
