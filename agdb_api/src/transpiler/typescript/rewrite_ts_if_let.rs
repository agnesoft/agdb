use crate::transpiler::rewrite::Rewrite;
use crate::transpiler::rewrite::RewriteContext;
use agdb::type_def::Expression;
use agdb::type_def::LiteralValue;
use agdb::type_def::Op;

pub struct RewriteTsIfLet;

impl Rewrite for RewriteTsIfLet {
    fn name(&self) -> &str {
        "rewrite_ts_if_let"
    }

    fn rewrite_expr(&self, expr: Expression, _ctx: &RewriteContext) -> Expression {
        match expr {
            Expression::If {
                condition,
                then_branch,
                else_branch,
            } if is_let_pattern(&condition) => {
                desugar_if_let(*condition, *then_branch, else_branch.map(|e| *e))
            }
            other => other,
        }
    }
}

fn is_let_pattern(expr: &Expression) -> bool {
    matches!(expr, Expression::Let { name, .. } if is_destructuring_pattern(name))
}

fn is_destructuring_pattern(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::TupleStruct { .. }
            | Expression::Array(_)
            | Expression::StructPattern { .. }
    )
}

fn desugar_if_let(
    condition: Expression,
    then_branch: Expression,
    else_branch: Option<Expression>,
) -> Expression {
    if let Expression::Let { name, ty: _, value } = condition {
        let binding = extract_binding(&name);
        let value_expr = value.map(|v| *v).unwrap_or(Expression::Literal(LiteralValue::Unit));

        let binding = if ident_name(&binding) == ident_name(&value_expr) {
            rename_binding(binding)
        } else {
            binding
        };

        let option_binding = Expression::Ident(format!(
            "_{}_opt",
            ident_name(&binding).unwrap_or("v")
        ));

        let let_option = Expression::Let {
            name: Box::new(option_binding.clone()),
            ty: None,
            value: Some(Box::new(value_expr)),
        };

        let let_value = Expression::Let {
            name: Box::new(binding.clone()),
            ty: None,
            value: Some(Box::new(Expression::FieldAccess {
                base: Box::new(option_binding.clone()),
                field: "value".to_owned(),
            })),
        };

        let null_check = Expression::Binary {
            op: Op::Ne,
            left: Box::new(option_binding),
            right: Box::new(Expression::Ident("null".to_owned())),
        };

        let (then_branch, else_branch) =
            if let (Some(orig), Some(renamed)) = (ident_name(&extract_binding(&name)), ident_name(&binding)) {
                if orig != renamed {
                    (
                        rename_ident(then_branch, orig, renamed),
                        else_branch.map(|e| rename_ident(e, orig, renamed)),
                    )
                } else {
                    (then_branch, else_branch)
                }
            } else {
                (then_branch, else_branch)
            };

        let then_with_value = Expression::Block(vec![let_value, then_branch]);

        let if_stmt = Expression::If {
            condition: Box::new(null_check),
            then_branch: Box::new(then_with_value),
            else_branch: else_branch.map(Box::new),
        };

        Expression::Block(vec![let_option, if_stmt])
    } else {
        Expression::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        }
    }
}

fn ident_name(expr: &Expression) -> Option<&str> {
    match expr {
        Expression::Ident(name) => Some(name),
        _ => None,
    }
}

fn rename_binding(binding: Expression) -> Expression {
    match binding {
        Expression::Ident(name) => Expression::Ident(format!("_{name}")),
        other => other,
    }
}

fn rename_ident(expr: Expression, from: &str, to: &str) -> Expression {
    match expr {
        Expression::Ident(ref name) if name == from => Expression::Ident(to.to_owned()),
        Expression::Block(stmts) => {
            Expression::Block(stmts.into_iter().map(|s| rename_ident(s, from, to)).collect())
        }
        Expression::Call {
            recipient,
            function,
            args,
        } => Expression::Call {
            recipient: recipient.map(|r| Box::new(rename_ident(*r, from, to))),
            function: Box::new(rename_ident(*function, from, to)),
            args: args.into_iter().map(|a| rename_ident(a, from, to)).collect(),
        },
        Expression::FieldAccess { base, field } => Expression::FieldAccess {
            base: Box::new(rename_ident(*base, from, to)),
            field,
        },
        Expression::Assign { target, value } => Expression::Assign {
            target: Box::new(rename_ident(*target, from, to)),
            value: Box::new(rename_ident(*value, from, to)),
        },
        Expression::If {
            condition,
            then_branch,
            else_branch,
        } => Expression::If {
            condition: Box::new(rename_ident(*condition, from, to)),
            then_branch: Box::new(rename_ident(*then_branch, from, to)),
            else_branch: else_branch.map(|e| Box::new(rename_ident(*e, from, to))),
        },
        Expression::Return(inner) => {
            Expression::Return(inner.map(|e| Box::new(rename_ident(*e, from, to))))
        }
        Expression::Binary { op, left, right } => Expression::Binary {
            op,
            left: Box::new(rename_ident(*left, from, to)),
            right: Box::new(rename_ident(*right, from, to)),
        },
        other => other,
    }
}

fn extract_binding(name: &Expression) -> Expression {
    match name {
        Expression::TupleStruct { expressions, .. } => {
            if let Some(first) = expressions.first() {
                first.clone()
            } else {
                Expression::Ident("_".to_owned())
            }
        }
        Expression::Array(elems) => {
            if let Some(first) = elems.first() {
                first.clone()
            } else {
                Expression::Ident("_".to_owned())
            }
        }
        Expression::StructPattern { fields, .. } => {
            if let Some(first) = fields.first() {
                first.clone()
            } else {
                Expression::Ident("_".to_owned())
            }
        }
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use crate::transpiler::rewrite::RewritePipeline;
    use super::*;

    #[test]
    fn if_let_some_desugared() {
        let mut expr = Expression::If {
            condition: Box::new(Expression::Let {
                name: Box::new(Expression::TupleStruct {
                    name: Box::new(Expression::Path {
                        ident: "Some".to_owned(),
                        parent: None,
                        generics: vec![],
                    }),
                    expressions: vec![Expression::Ident("token".to_owned())],
                }),
                ty: None,
                value: Some(Box::new(Expression::Ident("self_token".to_owned()))),
            }),
            then_branch: Box::new(Expression::Block(vec![Expression::Ident(
                "use_token".to_owned(),
            )])),
            else_branch: None,
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsIfLet)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        match expr {
            Expression::Block(stmts) => {
                assert_eq!(stmts.len(), 2);
                assert!(matches!(&stmts[0], Expression::Let { .. }));
                assert!(matches!(&stmts[1], Expression::If { .. }));
            }
            _ => panic!("Expected Block, got {:?}", expr),
        }
    }

    #[test]
    fn if_let_with_else_desugared() {
        let mut expr = Expression::If {
            condition: Box::new(Expression::Let {
                name: Box::new(Expression::TupleStruct {
                    name: Box::new(Expression::Path {
                        ident: "Some".to_owned(),
                        parent: None,
                        generics: vec![],
                    }),
                    expressions: vec![Expression::Ident("x".to_owned())],
                }),
                ty: None,
                value: Some(Box::new(Expression::Ident("opt".to_owned()))),
            }),
            then_branch: Box::new(Expression::Ident("a".to_owned())),
            else_branch: Some(Box::new(Expression::Ident("b".to_owned()))),
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsIfLet)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        match expr {
            Expression::Block(stmts) => {
                assert_eq!(stmts.len(), 2);
                if let Expression::If { else_branch, .. } = &stmts[1] {
                    assert!(else_branch.is_some());
                } else {
                    panic!("Expected If");
                }
            }
            _ => panic!("Expected Block"),
        }
    }

    #[test]
    fn regular_if_unchanged() {
        let mut expr = Expression::If {
            condition: Box::new(Expression::Ident("cond".to_owned())),
            then_branch: Box::new(Expression::Ident("a".to_owned())),
            else_branch: None,
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsIfLet)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::If { .. }));
    }

    #[test]
    fn let_without_pattern_unchanged() {
        let mut expr = Expression::If {
            condition: Box::new(Expression::Let {
                name: Box::new(Expression::Ident("x".to_owned())),
                ty: None,
                value: Some(Box::new(Expression::Literal(LiteralValue::I32(5)))),
            }),
            then_branch: Box::new(Expression::Ident("a".to_owned())),
            else_branch: None,
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsIfLet)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::If { .. }));
    }
}
