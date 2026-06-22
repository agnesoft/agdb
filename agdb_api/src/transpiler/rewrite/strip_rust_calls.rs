use super::Rewrite;
use super::RewriteContext;
use agdb::type_def::Expression;
use agdb::type_def::LiteralValue;
use agdb::type_def::Op;

pub struct StripRustCalls;

impl Rewrite for StripRustCalls {
    fn name(&self) -> &str {
        "strip_rust_calls"
    }

    fn rewrite_expr(&self, expr: Expression, _ctx: &RewriteContext) -> Expression {
        match expr {
            Expression::Call {
                recipient: None,
                ref function,
                mut args,
            } if !args.is_empty() && is_stdlib_identity(function) => args.remove(0),

            Expression::Call {
                recipient: None,
                ref function,
                ..
            } if is_stdlib_delete(function) => Expression::Block(vec![]),

            Expression::Call {
                recipient: None,
                ref function,
                ..
            } if is_stdlib_default(function) => Expression::Literal(LiteralValue::Unit),

            Expression::Call {
                recipient: Some(recipient),
                ref function,
                args,
            } if args.is_empty() && is_option_unwrap(function) => Expression::FieldAccess {
                base: recipient,
                field: "value".to_owned(),
            },

            Expression::Call {
                recipient: Some(recipient),
                ref function,
                args,
            } if args.is_empty() && is_identity_method(function) => *recipient,

            Expression::Call {
                recipient: Some(recipient),
                ref function,
                args,
            } if is_strip_with_args(function, &args) => *recipient,

            Expression::Call {
                recipient: Some(_),
                ref function,
                args,
            } if args.len() == 1 && is_method(function, "reserve") => Expression::Block(vec![]),

            Expression::Call {
                recipient: Some(recipient),
                ref function,
                mut args,
            } if args.len() == 1 && is_method(function, "unwrap_or") => Expression::Binary {
                op: Op::Or,
                left: recipient,
                right: Box::new(args.remove(0)),
            },

            other => other,
        }
    }
}

fn is_identity_method(function: &Expression) -> bool {
    match function {
        Expression::Ident(name) => matches!(
            name.as_str(),
            "clone" | "iter" | "to_string" | "to_owned" | "expect"
        ),
        Expression::Path {
            ident,
            parent: None,
            ..
        } => matches!(
            ident.as_str(),
            "clone" | "iter" | "to_string" | "to_owned" | "expect"
        ),
        _ => false,
    }
}

fn is_option_unwrap(function: &Expression) -> bool {
    is_method(function, "unwrap") || is_method(function, "unwrap_or_default")
}

fn is_strip_with_args(function: &Expression, args: &[Expression]) -> bool {
    if args.len() == 1 {
        is_method(function, "expect")
    } else {
        false
    }
}

fn is_method(function: &Expression, name: &str) -> bool {
    matches!(function, Expression::Ident(ident) if ident == name)
        || matches!(function, Expression::Path { ident, parent: None, .. } if ident == name)
}

fn is_stdlib_identity(function: &Expression) -> bool {
    if let Expression::Path { ident, parent: Some(parent), .. } = function {
        if matches!(ident.as_str(), "from_value" | "to_value" | "take") {
            return is_stdlib_path(parent);
        }
    }
    false
}

fn is_stdlib_path(expr: &Expression) -> bool {
    match expr {
        Expression::Ident(name) => name == "serde_json" || name == "std",
        Expression::Path { ident, parent: None, .. } => ident == "serde_json" || ident == "std",
        Expression::Path { parent: Some(parent), .. } => is_stdlib_path(parent),
        _ => false,
    }
}

fn is_stdlib_delete(function: &Expression) -> bool {
    if let Expression::Path { ident, parent: Some(parent), .. } = function {
        if ident == "swap" {
            return is_stdlib_path(parent);
        }
    }
    false
}

fn is_stdlib_default(function: &Expression) -> bool {
    if let Expression::Path { ident, parent: Some(parent), .. } = function {
        if ident == "default" {
            return is_stdlib_path(parent);
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use agdb::type_def::LiteralValue;
    use super::super::RewritePipeline;
    use super::*;

    #[test]
    fn clone_stripped() {
        let mut expr = Expression::Call {
            recipient: Some(Box::new(Expression::Ident("x".to_owned()))),
            function: Box::new(Expression::Ident("clone".to_owned())),
            args: vec![],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(StripRustCalls)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Ident(ref s) if s == "x"));
    }

    #[test]
    fn to_string_stripped() {
        let mut expr = Expression::Call {
            recipient: Some(Box::new(Expression::Ident("val".to_owned()))),
            function: Box::new(Expression::Ident("to_string".to_owned())),
            args: vec![],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(StripRustCalls)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Ident(ref s) if s == "val"));
    }

    #[test]
    fn reserve_deleted() {
        let mut expr = Expression::Call {
            recipient: Some(Box::new(Expression::Ident("vec".to_owned()))),
            function: Box::new(Expression::Ident("reserve".to_owned())),
            args: vec![Expression::Literal(LiteralValue::I32(10))],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(StripRustCalls)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Block(ref v) if v.is_empty()));
    }

    #[test]
    fn unwrap_or_becomes_or() {
        let mut expr = Expression::Call {
            recipient: Some(Box::new(Expression::Ident("opt".to_owned()))),
            function: Box::new(Expression::Ident("unwrap_or".to_owned())),
            args: vec![Expression::Literal(LiteralValue::I32(0))],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(StripRustCalls)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Binary { op: Op::Or, .. }));
    }

    #[test]
    fn expect_stripped() {
        let mut expr = Expression::Call {
            recipient: Some(Box::new(Expression::Ident("val".to_owned()))),
            function: Box::new(Expression::Ident("expect".to_owned())),
            args: vec![],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(StripRustCalls)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Ident(ref s) if s == "val"));
    }

    #[test]
    fn unrelated_method_unchanged() {
        let mut expr = Expression::Call {
            recipient: Some(Box::new(Expression::Ident("x".to_owned()))),
            function: Box::new(Expression::Ident("push".to_owned())),
            args: vec![Expression::Literal(LiteralValue::I32(1))],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(StripRustCalls)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Call { .. }));
    }
}
