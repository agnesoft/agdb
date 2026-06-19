use crate::transpiler::rewrite::Rewrite;
use crate::transpiler::rewrite::RewriteContext;
use agdb::type_def::Expression;
use agdb::type_def::Pattern;

pub struct RewriteTsIdentifiers;

impl Rewrite for RewriteTsIdentifiers {
    fn name(&self) -> &str {
        "rewrite_ts_identifiers"
    }

    fn rewrite_expr(&self, expr: Expression, ctx: &RewriteContext) -> Expression {
        match expr {
            Expression::Ident(ref name) if name == "self" => {
                Expression::Ident("this".to_owned())
            }
            Expression::Path {
                ref ident,
                parent,
                generics,
            } if ident == "Self" => Expression::Path {
                ident: ctx
                    .current_type
                    .clone()
                    .unwrap_or_else(|| "Self".to_owned()),
                parent,
                generics,
            },
            other => other,
        }
    }

    fn rewrite_pattern(&self, pattern: Pattern, _ctx: &RewriteContext) -> Pattern {
        match pattern {
            Pattern::Constructor { ref name, fields } if name.starts_with("Atomic") => {
                if let Some(first) = fields.into_iter().next() {
                    first
                } else {
                    Pattern::Constructor {
                        name: name.clone(),
                        fields: vec![],
                    }
                }
            }
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::transpiler::rewrite::RewritePipeline;
    use super::*;

    #[test]
    fn self_becomes_this() {
        let mut expr = Expression::Ident("self".to_owned());

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsIdentifiers)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Ident(ref s) if s == "this"));
    }

    #[test]
    fn self_type_becomes_class_name() {
        let mut expr = Expression::Path {
            ident: "Self".to_owned(),
            parent: None,
            generics: vec![],
        };

        let ctx = RewriteContext {
            current_type: Some("MyClass".to_owned()),
            ..Default::default()
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsIdentifiers)]);
        pipeline.rewrite_expr(&mut expr, &ctx);

        match expr {
            Expression::Path { ident, .. } => assert_eq!(ident, "MyClass"),
            _ => panic!("Expected Path, got {:?}", expr),
        }
    }

    #[test]
    fn self_type_without_context_stays() {
        let mut expr = Expression::Path {
            ident: "Self".to_owned(),
            parent: None,
            generics: vec![],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsIdentifiers)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        match expr {
            Expression::Path { ident, .. } => assert_eq!(ident, "Self"),
            _ => panic!("Expected Path, got {:?}", expr),
        }
    }

    #[test]
    fn other_ident_unchanged() {
        let mut expr = Expression::Ident("foo".to_owned());

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsIdentifiers)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Ident(ref s) if s == "foo"));
    }

    #[test]
    fn atomic_constructor_pattern_stripped() {
        let mut pattern = Pattern::Constructor {
            name: "AtomicU16".to_owned(),
            fields: vec![Pattern::Ident("value".to_owned())],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsIdentifiers)]);
        pipeline.rewrite_pattern(&mut pattern, &RewriteContext::default());

        assert!(matches!(pattern, Pattern::Ident(ref s) if s == "value"));
    }

    #[test]
    fn non_atomic_constructor_unchanged() {
        let mut pattern = Pattern::Constructor {
            name: "Option".to_owned(),
            fields: vec![Pattern::Ident("x".to_owned())],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsIdentifiers)]);
        pipeline.rewrite_pattern(&mut pattern, &RewriteContext::default());

        assert!(matches!(pattern, Pattern::Constructor { ref name, .. } if name == "Option"));
    }
}
