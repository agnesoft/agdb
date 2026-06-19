mod strip_atomics;
mod strip_memory_management;
mod strip_references;
mod strip_smart_pointers;

use agdb::type_def::Expression;
use agdb::type_def::Type;
pub use strip_atomics::StripAtomics;
pub use strip_memory_management::StripMemoryManagement;
pub use strip_references::StripReferences;
pub use strip_smart_pointers::StripSmartPointers;

#[derive(Debug, Default, Clone)]
pub struct RewriteContext {
    pub current_type: Option<String>,
    pub current_function: Option<String>,
    pub error_type: Option<String>,
}

pub trait Rewrite {
    fn name(&self) -> &str;

    fn rewrite_expr(&self, expr: Expression, ctx: &RewriteContext) -> Expression {
        let _ = ctx;
        expr
    }

    fn rewrite_type(&self, ty: Type, ctx: &RewriteContext) -> Type {
        let _ = ctx;
        ty
    }
}

pub struct RewritePipeline {
    rewrites: Vec<Box<dyn Rewrite>>,
}

impl RewritePipeline {
    pub fn new(rewrites: Vec<Box<dyn Rewrite>>) -> Self {
        Self { rewrites }
    }

    pub fn rewrite_expr(&self, expr: &mut Expression, ctx: &RewriteContext) {
        for rewrite in &self.rewrites {
            let taken = std::mem::take(expr);
            *expr = walk_expr(taken, rewrite.as_ref(), ctx);
        }
    }

    pub fn rewrite_exprs(&self, exprs: &mut [Expression], ctx: &RewriteContext) {
        for expr in exprs.iter_mut() {
            self.rewrite_expr(expr, ctx);
        }
    }

    pub fn rewrite_type(&self, ty: &mut Type, ctx: &RewriteContext) {
        for rewrite in &self.rewrites {
            let taken = std::mem::take(ty);
            *ty = rewrite.rewrite_type(taken, ctx);
        }
    }
}

fn walk_expr(expr: Expression, rewrite: &dyn Rewrite, ctx: &RewriteContext) -> Expression {
    let walked = match expr {
        Expression::Array(elems) => Expression::Array(
            elems
                .into_iter()
                .map(|e| walk_expr(e, rewrite, ctx))
                .collect(),
        ),
        Expression::Assign { target, value } => Expression::Assign {
            target: Box::new(walk_expr(*target, rewrite, ctx)),
            value: Box::new(walk_expr(*value, rewrite, ctx)),
        },
        Expression::Await(inner) => Expression::Await(Box::new(walk_expr(*inner, rewrite, ctx))),
        Expression::Binary { op, left, right } => Expression::Binary {
            op,
            left: Box::new(walk_expr(*left, rewrite, ctx)),
            right: Box::new(walk_expr(*right, rewrite, ctx)),
        },
        Expression::Block(stmts) => Expression::Block(
            stmts
                .into_iter()
                .map(|s| walk_expr(s, rewrite, ctx))
                .collect(),
        ),
        Expression::Call {
            recipient,
            function,
            args,
        } => Expression::Call {
            recipient: recipient.map(|r| Box::new(walk_expr(*r, rewrite, ctx))),
            function: Box::new(walk_expr(*function, rewrite, ctx)),
            args: args
                .into_iter()
                .map(|a| walk_expr(a, rewrite, ctx))
                .collect(),
        },
        Expression::Closure(mut f) => {
            f.body = f
                .body
                .into_iter()
                .map(|e| walk_expr(e, rewrite, ctx))
                .collect();
            Expression::Closure(f)
        }
        Expression::FieldAccess { base, field } => Expression::FieldAccess {
            base: Box::new(walk_expr(*base, rewrite, ctx)),
            field,
        },
        Expression::For {
            pattern,
            iterable,
            body,
        } => Expression::For {
            pattern: Box::new(walk_expr(*pattern, rewrite, ctx)),
            iterable: Box::new(walk_expr(*iterable, rewrite, ctx)),
            body: Box::new(walk_expr(*body, rewrite, ctx)),
        },
        Expression::Format {
            format_string,
            args,
        } => Expression::Format {
            format_string,
            args: args
                .into_iter()
                .map(|a| walk_expr(a, rewrite, ctx))
                .collect(),
        },
        Expression::If {
            condition,
            then_branch,
            else_branch,
        } => Expression::If {
            condition: Box::new(walk_expr(*condition, rewrite, ctx)),
            then_branch: Box::new(walk_expr(*then_branch, rewrite, ctx)),
            else_branch: else_branch.map(|eb| Box::new(walk_expr(*eb, rewrite, ctx))),
        },
        Expression::Index { base, index } => Expression::Index {
            base: Box::new(walk_expr(*base, rewrite, ctx)),
            index: Box::new(walk_expr(*index, rewrite, ctx)),
        },
        Expression::Let { name, ty, value } => Expression::Let {
            name: Box::new(walk_expr(*name, rewrite, ctx)),
            ty,
            value: value.map(|v| Box::new(walk_expr(*v, rewrite, ctx))),
        },
        Expression::Match { scrutinee, arms } => Expression::Match {
            scrutinee: Box::new(walk_expr(*scrutinee, rewrite, ctx)),
            arms: arms
                .into_iter()
                .map(|mut arm| {
                    arm.guard = arm.guard.map(|g| Box::new(walk_expr(*g, rewrite, ctx)));
                    arm.body = Box::new(walk_expr(*arm.body, rewrite, ctx));
                    arm
                })
                .collect(),
        },
        Expression::Path {
            ident,
            parent,
            generics,
        } => Expression::Path {
            ident,
            parent: parent.map(|p| Box::new(walk_expr(*p, rewrite, ctx))),
            generics,
        },
        Expression::Range {
            start,
            end,
            inclusive,
        } => Expression::Range {
            start: start.map(|s| Box::new(walk_expr(*s, rewrite, ctx))),
            end: end.map(|e| Box::new(walk_expr(*e, rewrite, ctx))),
            inclusive,
        },
        Expression::Reference(inner) => {
            Expression::Reference(Box::new(walk_expr(*inner, rewrite, ctx)))
        }
        Expression::Return(inner) => {
            Expression::Return(inner.map(|i| Box::new(walk_expr(*i, rewrite, ctx))))
        }
        Expression::Struct { name, fields } => Expression::Struct {
            name: Box::new(walk_expr(*name, rewrite, ctx)),
            fields: fields
                .into_iter()
                .map(|(k, v)| (k, walk_expr(v, rewrite, ctx)))
                .collect(),
        },
        Expression::StructPattern { name, fields } => Expression::StructPattern {
            name: Box::new(walk_expr(*name, rewrite, ctx)),
            fields: fields
                .into_iter()
                .map(|e| walk_expr(e, rewrite, ctx))
                .collect(),
        },
        Expression::Try(inner) => Expression::Try(Box::new(walk_expr(*inner, rewrite, ctx))),
        Expression::Tuple(elems) => Expression::Tuple(
            elems
                .into_iter()
                .map(|e| walk_expr(e, rewrite, ctx))
                .collect(),
        ),
        Expression::TupleStruct { name, expressions } => Expression::TupleStruct {
            name: Box::new(walk_expr(*name, rewrite, ctx)),
            expressions: expressions
                .into_iter()
                .map(|e| walk_expr(e, rewrite, ctx))
                .collect(),
        },
        Expression::TupleAccess { base, index } => Expression::TupleAccess {
            base: Box::new(walk_expr(*base, rewrite, ctx)),
            index,
        },
        Expression::Unary { op, expr } => Expression::Unary {
            op,
            expr: Box::new(walk_expr(*expr, rewrite, ctx)),
        },
        Expression::While { condition, body } => Expression::While {
            condition: Box::new(walk_expr(*condition, rewrite, ctx)),
            body: Box::new(walk_expr(*body, rewrite, ctx)),
        },
        leaf @ (Expression::Break
        | Expression::Continue
        | Expression::Ident(_)
        | Expression::Literal(_)
        | Expression::Wild) => leaf,
    };

    rewrite.rewrite_expr(walked, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agdb::type_def::{LiteralValue, Op};

    struct DoubleLiterals;

    impl Rewrite for DoubleLiterals {
        fn name(&self) -> &str {
            "double_literals"
        }

        fn rewrite_expr(&self, expr: Expression, _ctx: &RewriteContext) -> Expression {
            match expr {
                Expression::Literal(LiteralValue::I32(n)) => {
                    Expression::Literal(LiteralValue::I32(n * 2))
                }
                other => other,
            }
        }
    }

    struct StripRef;

    impl Rewrite for StripRef {
        fn name(&self) -> &str {
            "strip_ref"
        }

        fn rewrite_expr(&self, expr: Expression, _ctx: &RewriteContext) -> Expression {
            match expr {
                Expression::Reference(inner) => *inner,
                other => other,
            }
        }
    }

    #[test]
    fn identity_rewrite_is_noop() {
        struct Identity;
        impl Rewrite for Identity {
            fn name(&self) -> &str {
                "identity"
            }
        }

        let mut expr = Expression::Literal(LiteralValue::I32(42));
        let pipeline = RewritePipeline::new(vec![Box::new(Identity)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());
        assert!(matches!(expr, Expression::Literal(LiteralValue::I32(42))));
    }

    #[test]
    fn rewrite_transforms_leaf() {
        let mut expr = Expression::Literal(LiteralValue::I32(5));
        let pipeline = RewritePipeline::new(vec![Box::new(DoubleLiterals)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());
        assert!(matches!(expr, Expression::Literal(LiteralValue::I32(10))));
    }

    #[test]
    fn rewrite_walks_children_bottom_up() {
        let mut expr = Expression::Binary {
            op: Op::Add,
            left: Box::new(Expression::Literal(LiteralValue::I32(3))),
            right: Box::new(Expression::Literal(LiteralValue::I32(7))),
        };
        let pipeline = RewritePipeline::new(vec![Box::new(DoubleLiterals)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        match expr {
            Expression::Binary { left, right, .. } => {
                assert!(matches!(*left, Expression::Literal(LiteralValue::I32(6))));
                assert!(matches!(*right, Expression::Literal(LiteralValue::I32(14))));
            }
            _ => panic!("Expected Binary, got {:?}", expr),
        }
    }

    #[test]
    fn pipeline_applies_rewrites_in_order() {
        let mut expr = Expression::Reference(Box::new(Expression::Literal(LiteralValue::I32(3))));
        let pipeline = RewritePipeline::new(vec![Box::new(StripRef), Box::new(DoubleLiterals)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());
        assert!(matches!(expr, Expression::Literal(LiteralValue::I32(6))));
    }

    #[test]
    fn pipeline_order_matters() {
        let mut expr = Expression::Reference(Box::new(Expression::Literal(LiteralValue::I32(3))));
        let pipeline = RewritePipeline::new(vec![Box::new(DoubleLiterals), Box::new(StripRef)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());
        assert!(matches!(expr, Expression::Literal(LiteralValue::I32(6))));
    }

    #[test]
    fn rewrite_exprs_rewrites_all() {
        let mut body = vec![
            Expression::Literal(LiteralValue::I32(1)),
            Expression::Literal(LiteralValue::I32(2)),
            Expression::Literal(LiteralValue::I32(3)),
        ];
        let pipeline = RewritePipeline::new(vec![Box::new(DoubleLiterals)]);
        pipeline.rewrite_exprs(&mut body, &RewriteContext::default());

        assert!(matches!(body[0], Expression::Literal(LiteralValue::I32(2))));
        assert!(matches!(body[1], Expression::Literal(LiteralValue::I32(4))));
        assert!(matches!(body[2], Expression::Literal(LiteralValue::I32(6))));
    }
}
