use super::Rewrite;
use super::RewriteContext;
use agdb::type_def::Expression;

pub struct StripMemoryManagement;

impl Rewrite for StripMemoryManagement {
    fn name(&self) -> &str {
        "strip_memory_management"
    }

    fn rewrite_expr(&self, expr: Expression, _ctx: &RewriteContext) -> Expression {
        match expr {
            Expression::Call {
                recipient: Some(recipient),
                ref function,
                args,
            } if args.is_empty() && is_strippable_method(function) => *recipient,
            other => other,
        }
    }
}

fn is_strippable_method(function: &Expression) -> bool {
    let name = match function {
        Expression::Path {
            ident,
            parent: None,
            ..
        } => ident.as_str(),
        Expression::Ident(ident) => ident.as_str(),
        _ => return false,
    };
    matches!(name, "write" | "upgrade")
}

#[cfg(test)]
mod tests {
    use super::super::RewritePipeline;
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
    fn write_returns_recipient() {
        let mut expr = method_call("lock", "write", vec![]);

        let pipeline = RewritePipeline::new(vec![Box::new(StripMemoryManagement)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Ident(ref s) if s == "lock"));
    }

    #[test]
    fn upgrade_returns_recipient() {
        let mut expr = method_call("weak_ref", "upgrade", vec![]);

        let pipeline = RewritePipeline::new(vec![Box::new(StripMemoryManagement)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Ident(ref s) if s == "weak_ref"));
    }

    #[test]
    fn write_with_args_unchanged() {
        let mut expr = method_call("file", "write", vec![Expression::Ident("data".to_owned())]);

        let pipeline = RewritePipeline::new(vec![Box::new(StripMemoryManagement)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Call { .. }));
    }

    #[test]
    fn unrelated_method_unchanged() {
        let mut expr = method_call("vec", "push", vec![Expression::Ident("item".to_owned())]);

        let pipeline = RewritePipeline::new(vec![Box::new(StripMemoryManagement)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Call { .. }));
    }
}
