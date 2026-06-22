use crate::transpiler::rewrite::Rewrite;
use crate::transpiler::rewrite::RewriteContext;
use agdb::type_def::Expression;

pub struct RewriteTsInto;

impl Rewrite for RewriteTsInto {
    fn name(&self) -> &str {
        "rewrite_ts_into"
    }

    fn rewrite_expr(&self, expr: Expression, ctx: &RewriteContext) -> Expression {
        match expr {
            Expression::TupleAccess { ref base, index: 0 } if is_into_call(base) => {
                extract_into_arg(base)
            }
            Expression::FieldAccess { ref base, ref field }
                if field == "_0" && is_into_call(base) =>
            {
                extract_into_arg(base)
            }
            Expression::Call {
                recipient: Some(ref base),
                ref function,
                ..
            } if is_method_named(function, "get_ids") && is_into_call(base) => {
                extract_into_arg(base)
            }
            Expression::Call {
                recipient: Some(recipient),
                ref function,
                args,
            } if is_method_named(function, "into") && args.is_empty() => {
                rewrite_bare_into(*recipient, ctx)
            }
            other => other,
        }
    }
}

fn is_into_call(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::Call {
            recipient: None,
            function,
            args,
        } if args.len() == 1 && is_into_function(function)
    )
}

fn is_into_function(function: &Expression) -> bool {
    match function {
        Expression::Path {
            ident,
            parent: Some(parent),
            ..
        } if ident == "into" => is_into_path(parent),
        _ => false,
    }
}

fn is_into_path(expr: &Expression) -> bool {
    match expr {
        Expression::Ident(name) => name == "Into",
        Expression::Path {
            ident, parent: None, ..
        } => ident == "Into",
        _ => false,
    }
}

fn extract_into_arg(call: &Expression) -> Expression {
    match call {
        Expression::Call { args, .. } if !args.is_empty() => args[0].clone(),
        _ => unreachable!("is_into_call guarantees this is a Call with 1 arg"),
    }
}

fn is_method_named(function: &Expression, name: &str) -> bool {
    matches!(function, Expression::Ident(ident) if ident == name)
        || matches!(function, Expression::Path { ident, parent: None, .. } if ident == name)
}

fn rewrite_bare_into(recipient: Expression, ctx: &RewriteContext) -> Expression {
    let param_name = match &recipient {
        Expression::Ident(name) => Some(name.as_str()),
        _ => None,
    };

    if let Some(name) = param_name {
        if let Some(target) = ctx.into_targets.get(name) {
            if target == "QueryIds" {
                return Expression::Call {
                    recipient: None,
                    function: Box::new(Expression::Path {
                        ident: "Ids".to_owned(),
                        parent: Some(Box::new(Expression::Ident("QueryIds".to_owned()))),
                        generics: vec![],
                    }),
                    args: vec![recipient],
                };
            }
        }
    }

    recipient
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::transpiler::rewrite::RewritePipeline;
    use super::*;

    #[test]
    fn into_call_with_tuple_access_stripped() {
        let mut expr = Expression::TupleAccess {
            base: Box::new(Expression::Call {
                recipient: None,
                function: Box::new(Expression::Path {
                    ident: "into".to_owned(),
                    parent: Some(Box::new(Expression::Ident("Into".to_owned()))),
                    generics: vec![],
                }),
                args: vec![Expression::Ident("names".to_owned())],
            }),
            index: 0,
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsInto)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Ident(ref s) if s == "names"));
    }

    #[test]
    fn into_call_with_field_access_stripped() {
        let mut expr = Expression::FieldAccess {
            base: Box::new(Expression::Call {
                recipient: None,
                function: Box::new(Expression::Path {
                    ident: "into".to_owned(),
                    parent: Some(Box::new(Expression::Ident("Into".to_owned()))),
                    generics: vec![],
                }),
                args: vec![Expression::Ident("keys".to_owned())],
            }),
            field: "_0".to_owned(),
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsInto)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Ident(ref s) if s == "keys"));
    }

    #[test]
    fn bare_into_with_query_ids_target_wraps() {
        let mut expr = Expression::Call {
            recipient: Some(Box::new(Expression::Ident("ids".to_owned()))),
            function: Box::new(Expression::Ident("into".to_owned())),
            args: vec![],
        };

        let mut into_targets = HashMap::new();
        into_targets.insert("ids".to_owned(), "QueryIds".to_owned());
        let ctx = RewriteContext {
            into_targets,
            ..Default::default()
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsInto)]);
        pipeline.rewrite_expr(&mut expr, &ctx);

        match expr {
            Expression::Call {
                recipient: None,
                function,
                args,
            } => {
                assert!(matches!(*function, Expression::Path { ref ident, .. } if ident == "Ids"));
                assert_eq!(args.len(), 1);
                assert!(matches!(&args[0], Expression::Ident(s) if s == "ids"));
            }
            _ => panic!("Expected Call, got {:?}", expr),
        }
    }

    #[test]
    fn bare_into_without_query_ids_strips() {
        let mut expr = Expression::Call {
            recipient: Some(Box::new(Expression::Ident("names".to_owned()))),
            function: Box::new(Expression::Ident("into".to_owned())),
            args: vec![],
        };

        let mut into_targets = HashMap::new();
        into_targets.insert("names".to_owned(), "QueryAliases".to_owned());
        let ctx = RewriteContext {
            into_targets,
            ..Default::default()
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsInto)]);
        pipeline.rewrite_expr(&mut expr, &ctx);

        assert!(matches!(expr, Expression::Ident(ref s) if s == "names"));
    }

    #[test]
    fn bare_into_without_context_strips() {
        let mut expr = Expression::Call {
            recipient: Some(Box::new(Expression::Ident("x".to_owned()))),
            function: Box::new(Expression::Ident("into".to_owned())),
            args: vec![],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsInto)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Ident(ref s) if s == "x"));
    }

    #[test]
    fn get_ids_on_into_call_stripped() {
        let mut expr = Expression::Call {
            recipient: Some(Box::new(Expression::Call {
                recipient: None,
                function: Box::new(Expression::Path {
                    ident: "into".to_owned(),
                    parent: Some(Box::new(Expression::Ident("Into".to_owned()))),
                    generics: vec![],
                }),
                args: vec![Expression::Ident("ids".to_owned())],
            })),
            function: Box::new(Expression::Ident("get_ids".to_owned())),
            args: vec![],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsInto)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Ident(ref s) if s == "ids"));
    }
}
