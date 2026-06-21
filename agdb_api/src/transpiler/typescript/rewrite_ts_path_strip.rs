use crate::transpiler::rewrite::Rewrite;
use crate::transpiler::rewrite::RewriteContext;
use agdb::type_def::Expression;

pub struct RewriteTsPathStrip;

impl Rewrite for RewriteTsPathStrip {
    fn name(&self) -> &str {
        "rewrite_ts_path_strip"
    }

    fn rewrite_expr(&self, expr: Expression, _ctx: &RewriteContext) -> Expression {
        match expr {
            Expression::Path {
                ident,
                parent: Some(parent),
                generics,
            } if is_crate(&parent) => Expression::Path {
                ident,
                parent: None,
                generics,
            },
            other => other,
        }
    }
}

fn is_crate(expr: &Expression) -> bool {
    matches!(expr, Expression::Ident(name) if name == "crate")
        || matches!(expr, Expression::Path { ident, parent: None, .. } if ident == "crate")
}

#[cfg(test)]
mod tests {
    use crate::transpiler::rewrite::RewritePipeline;
    use super::*;

    #[test]
    fn crate_prefix_stripped() {
        let mut expr = Expression::Path {
            ident: "SearchQuery".to_owned(),
            parent: Some(Box::new(Expression::Ident("crate".to_owned()))),
            generics: vec![],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsPathStrip)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        match expr {
            Expression::Path {
                ident, parent, ..
            } => {
                assert_eq!(ident, "SearchQuery");
                assert!(parent.is_none());
            }
            _ => panic!("Expected Path, got {:?}", expr),
        }
    }

    #[test]
    fn non_crate_path_unchanged() {
        let mut expr = Expression::Path {
            ident: "new".to_owned(),
            parent: Some(Box::new(Expression::Ident("MyClass".to_owned()))),
            generics: vec![],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsPathStrip)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        match expr {
            Expression::Path { parent, .. } => {
                assert!(parent.is_some());
            }
            _ => panic!("Expected Path"),
        }
    }
}
