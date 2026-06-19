use super::Rewrite;
use super::RewriteContext;
use agdb::type_def::Expression;

pub struct StripSmartPointers;

impl Rewrite for StripSmartPointers {
    fn name(&self) -> &str {
        "strip_smart_pointers"
    }

    fn rewrite_expr(&self, expr: Expression, _ctx: &RewriteContext) -> Expression {
        match expr {
            Expression::Call {
                recipient: None,
                ref function,
                mut args,
            } if args.len() == 1 && is_smart_pointer_constructor(function) => args.remove(0),
            other => other,
        }
    }
}

fn is_smart_pointer_constructor(function: &Expression) -> bool {
    matches!(
        function,
        Expression::Path {
            ident,
            parent: Some(parent),
            ..
        } if (ident == "new" || ident == "downgrade")
            && matches!(
                parent.as_ref(),
                Expression::Ident(name) | Expression::Path { ident: name, .. }
                    if name == "Arc" || name == "Box"
            )
    )
}

#[cfg(test)]
mod tests {
    use super::super::RewritePipeline;
    use super::*;
    use agdb::type_def::LiteralValue;

    fn make_smart_ptr_call(ty: &str, method: &str, arg: Expression) -> Expression {
        Expression::Call {
            recipient: None,
            function: Box::new(Expression::Path {
                ident: method.to_owned(),
                parent: Some(Box::new(Expression::Ident(ty.to_owned()))),
                generics: vec![],
            }),
            args: vec![arg],
        }
    }

    #[test]
    fn arc_new_stripped() {
        let inner = Expression::Literal(LiteralValue::I32(42));
        let mut expr = make_smart_ptr_call("Arc", "new", inner);

        let pipeline = RewritePipeline::new(vec![Box::new(StripSmartPointers)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Literal(LiteralValue::I32(42))));
    }

    #[test]
    fn arc_downgrade_stripped() {
        let inner = Expression::Ident("x".to_owned());
        let mut expr = make_smart_ptr_call("Arc", "downgrade", inner);

        let pipeline = RewritePipeline::new(vec![Box::new(StripSmartPointers)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Ident(ref s) if s == "x"));
    }

    #[test]
    fn box_new_stripped() {
        let inner = Expression::Literal(LiteralValue::I32(99));
        let mut expr = make_smart_ptr_call("Box", "new", inner);

        let pipeline = RewritePipeline::new(vec![Box::new(StripSmartPointers)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Literal(LiteralValue::I32(99))));
    }

    #[test]
    fn unrelated_call_unchanged() {
        let mut expr = Expression::Call {
            recipient: None,
            function: Box::new(Expression::Path {
                ident: "new".to_owned(),
                parent: Some(Box::new(Expression::Ident("HashMap".to_owned()))),
                generics: vec![],
            }),
            args: vec![],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(StripSmartPointers)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Call { .. }));
    }
}
