use crate::transpiler::rewrite::Rewrite;
use crate::transpiler::rewrite::RewriteContext;
use agdb::type_def::Expression;
use agdb::type_def::LiteralValue;
use agdb::type_def::Op;

pub struct RewriteTsMethods;

impl Rewrite for RewriteTsMethods {
    fn name(&self) -> &str {
        "rewrite_ts_methods"
    }

    fn rewrite_expr(&self, expr: Expression, _ctx: &RewriteContext) -> Expression {
        match expr {
            Expression::Call {
                recipient: Some(recipient),
                ref function,
                args,
            } if args.is_empty() && is_method(function, "is_empty") => Expression::Binary {
                op: Op::Eq,
                left: Box::new(Expression::FieldAccess {
                    base: recipient,
                    field: "length".to_owned(),
                }),
                right: Box::new(Expression::Literal(LiteralValue::I32(0))),
            },

            Expression::Call {
                recipient: Some(recipient),
                ref function,
                args,
            } if args.is_empty() && is_method(function, "len") => Expression::FieldAccess {
                base: recipient,
                field: "length".to_owned(),
            },

            Expression::Call {
                recipient: Some(recipient),
                ref function,
                ..
            } if is_method(function, "map_err") => *recipient,

            Expression::Call {
                recipient: Some(recipient),
                ref function,
                ..
            } if is_method(function, "get_or_init") => *recipient,

            other => other,
        }
    }
}

fn is_method(function: &Expression, name: &str) -> bool {
    matches!(
        function,
        Expression::Path { ident, parent: None, .. } if ident == name
    ) || matches!(
        function,
        Expression::Ident(ident) if ident == name
    )
}

#[cfg(test)]
mod tests {
    use crate::transpiler::rewrite::RewritePipeline;
    use super::*;

    fn method_call(recipient: &str, method: &str, args: Vec<Expression>) -> Expression {
        Expression::Call {
            recipient: Some(Box::new(Expression::Ident(recipient.to_owned()))),
            function: Box::new(Expression::Path {
                ident: method.to_owned(),
                parent: None,
                generics: vec![],
            }),
            args,
        }
    }

    #[test]
    fn is_empty_becomes_length_eq_zero() {
        let mut expr = method_call("vec", "is_empty", vec![]);

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsMethods)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        match expr {
            Expression::Binary { op, left, right } => {
                assert!(matches!(op, Op::Eq));
                assert!(matches!(*left, Expression::FieldAccess { ref field, .. } if field == "length"));
                assert!(matches!(*right, Expression::Literal(LiteralValue::I32(0))));
            }
            _ => panic!("Expected Binary, got {:?}", expr),
        }
    }

    #[test]
    fn len_becomes_length() {
        let mut expr = method_call("vec", "len", vec![]);

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsMethods)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::FieldAccess { ref field, .. } if field == "length"));
    }

    #[test]
    fn map_err_returns_recipient() {
        let mut expr = method_call(
            "result",
            "map_err",
            vec![Expression::Ident("handler".to_owned())],
        );

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsMethods)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Ident(ref s) if s == "result"));
    }

    #[test]
    fn get_or_init_returns_recipient() {
        let mut expr = method_call(
            "once_lock",
            "get_or_init",
            vec![Expression::Ident("init_fn".to_owned())],
        );

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsMethods)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Ident(ref s) if s == "once_lock"));
    }

    #[test]
    fn unrelated_method_unchanged() {
        let mut expr = method_call("vec", "push", vec![Expression::Ident("item".to_owned())]);

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsMethods)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Call { .. }));
    }
}
