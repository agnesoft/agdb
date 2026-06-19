use super::Rewrite;
use super::RewriteContext;
use agdb::type_def::Expression;

pub struct StripReferences;

impl Rewrite for StripReferences {
    fn name(&self) -> &str {
        "strip_references"
    }

    fn rewrite_expr(&self, expr: Expression, _ctx: &RewriteContext) -> Expression {
        match expr {
            Expression::Reference(inner) => *inner,
            Expression::Try(inner) => *inner,
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::RewritePipeline;
    use super::*;
    use agdb::type_def::LiteralValue;

    #[test]
    fn reference_stripped() {
        let mut expr = Expression::Reference(Box::new(Expression::Ident("x".to_owned())));

        let pipeline = RewritePipeline::new(vec![Box::new(StripReferences)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Ident(ref s) if s == "x"));
    }

    #[test]
    fn try_stripped() {
        let mut expr = Expression::Try(Box::new(Expression::Ident("result".to_owned())));

        let pipeline = RewritePipeline::new(vec![Box::new(StripReferences)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Ident(ref s) if s == "result"));
    }

    #[test]
    fn nested_reference_stripped_by_bottom_up() {
        let mut expr = Expression::Reference(Box::new(Expression::Reference(Box::new(
            Expression::Literal(LiteralValue::I32(1)),
        ))));

        let pipeline = RewritePipeline::new(vec![Box::new(StripReferences)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Literal(LiteralValue::I32(1))));
    }

    #[test]
    fn non_reference_unchanged() {
        let mut expr = Expression::Literal(LiteralValue::I32(42));

        let pipeline = RewritePipeline::new(vec![Box::new(StripReferences)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Literal(LiteralValue::I32(42))));
    }
}
