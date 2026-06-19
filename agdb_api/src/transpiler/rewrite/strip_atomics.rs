use super::Rewrite;
use super::RewriteContext;
use agdb::type_def::Expression;
use agdb::type_def::Op;

pub struct StripAtomics;

impl Rewrite for StripAtomics {
    fn name(&self) -> &str {
        "strip_atomics"
    }

    fn rewrite_expr(&self, expr: Expression, _ctx: &RewriteContext) -> Expression {
        match expr {
            Expression::Call {
                recipient: None,
                ref function,
                mut args,
            } if args.len() == 1 && is_atomic_constructor(function) => args.remove(0),

            Expression::Call {
                recipient: Some(recipient),
                ref function,
                mut args,
            } if !args.is_empty() && is_method(function, "fetch_add") => Expression::Binary {
                op: Op::AddAssign,
                left: recipient,
                right: Box::new(args.remove(0)),
            },

            other => other,
        }
    }
}

fn is_atomic_constructor(function: &Expression) -> bool {
    matches!(
        function,
        Expression::Path { ident, parent: Some(parent), .. }
            if ident == "new"
                && matches!(
                    parent.as_ref(),
                    Expression::Ident(name) | Expression::Path { ident: name, .. }
                        if name.starts_with("Atomic")
                )
    )
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
    use super::super::RewritePipeline;
    use super::*;
    use agdb::type_def::LiteralValue;

    #[test]
    fn atomic_new_stripped() {
        let mut expr = Expression::Call {
            recipient: None,
            function: Box::new(Expression::Path {
                ident: "new".to_owned(),
                parent: Some(Box::new(Expression::Path {
                    ident: "AtomicU16".to_owned(),
                    parent: None,
                    generics: vec![],
                })),
                generics: vec![],
            }),
            args: vec![Expression::Literal(LiteralValue::I32(0))],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(StripAtomics)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Literal(LiteralValue::I32(0))));
    }

    #[test]
    fn fetch_add_becomes_add_assign() {
        let mut expr = Expression::Call {
            recipient: Some(Box::new(Expression::Ident("counter".to_owned()))),
            function: Box::new(Expression::Path {
                ident: "fetch_add".to_owned(),
                parent: None,
                generics: vec![],
            }),
            args: vec![
                Expression::Literal(LiteralValue::I32(1)),
                Expression::Ident("Ordering_SeqCst".to_owned()),
            ],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(StripAtomics)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        match expr {
            Expression::Binary { op, left, right } => {
                assert!(matches!(op, Op::AddAssign));
                assert!(matches!(*left, Expression::Ident(ref s) if s == "counter"));
                assert!(matches!(*right, Expression::Literal(LiteralValue::I32(1))));
            }
            _ => panic!("Expected Binary, got {:?}", expr),
        }
    }
}
