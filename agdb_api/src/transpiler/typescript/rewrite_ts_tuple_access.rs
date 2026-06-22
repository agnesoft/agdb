use crate::transpiler::rewrite::Rewrite;
use crate::transpiler::rewrite::RewriteContext;
use agdb::type_def::Expression;

fn is_self_access(base: &Expression) -> bool {
    match base {
        Expression::Ident(name) => name == "self" || name == "this",
        Expression::FieldAccess { base, .. } => is_self_access(base),
        Expression::TupleAccess { base, .. } => is_self_access(base),
        _ => false,
    }
}

pub struct RewriteTsTupleAccess;

impl Rewrite for RewriteTsTupleAccess {
    fn name(&self) -> &str {
        "rewrite_ts_tuple_access"
    }

    fn rewrite_expr(&self, expr: Expression, _ctx: &RewriteContext) -> Expression {
        match expr {
            Expression::TupleAccess { ref base, index } if is_self_access(base) => {
                Expression::FieldAccess {
                    base: match expr {
                        Expression::TupleAccess { base, .. } => base,
                        _ => unreachable!(),
                    },
                    field: format!("_{index}"),
                }
            }
            Expression::TupleAccess { base, index } => Expression::Index {
                base,
                index: Box::new(Expression::Literal(
                    agdb::type_def::LiteralValue::I32(index as i32),
                )),
            },
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::transpiler::rewrite::RewritePipeline;
    use super::*;

    #[test]
    fn tuple_access_zero_becomes_field() {
        let mut expr = Expression::TupleAccess {
            base: Box::new(Expression::Ident("this".to_owned())),
            index: 0,
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsTupleAccess)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        match expr {
            Expression::FieldAccess { base, field } => {
                assert!(matches!(*base, Expression::Ident(ref s) if s == "this"));
                assert_eq!(field, "_0");
            }
            _ => panic!("Expected FieldAccess, got {:?}", expr),
        }
    }

    #[test]
    fn tuple_access_on_non_self_becomes_index() {
        let mut expr = Expression::TupleAccess {
            base: Box::new(Expression::Ident("result".to_owned())),
            index: 1,
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsTupleAccess)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Index { .. }));
    }

    #[test]
    fn non_tuple_access_unchanged() {
        let mut expr = Expression::FieldAccess {
            base: Box::new(Expression::Ident("x".to_owned())),
            field: "name".to_owned(),
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsTupleAccess)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::FieldAccess { ref field, .. } if field == "name"));
    }
}
