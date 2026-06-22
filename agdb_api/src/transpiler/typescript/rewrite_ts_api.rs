use crate::transpiler::rewrite::Rewrite;
use crate::transpiler::rewrite::RewriteContext;
use agdb::type_def::Expression;

pub struct RewriteTsApi;

impl Rewrite for RewriteTsApi {
    fn name(&self) -> &str {
        "rewrite_ts_api"
    }

    fn rewrite_expr(&self, expr: Expression, ctx: &RewriteContext) -> Expression {
        match expr {
            Expression::Call {
                recipient: None,
                ref function,
                mut args,
            } if args.len() == 1 && is_serde_json_from_str(function) => Expression::Call {
                recipient: None,
                function: Box::new(Expression::Path {
                    ident: "parse".to_owned(),
                    parent: Some(Box::new(Expression::Ident("JSON".to_owned()))),
                    generics: vec![],
                }),
                args: vec![args.remove(0)],
            },

            Expression::Call {
                recipient: None,
                ref function,
                args,
            } if args.len() == 1 && is_std_thread_sleep(function) => {
                Expression::Await(Box::new(Expression::Call {
                    recipient: None,
                    function: Box::new(Expression::Path {
                        ident: "sleep".to_owned(),
                        parent: Some(Box::new(Expression::Path {
                            ident: "thread".to_owned(),
                            parent: Some(Box::new(Expression::Ident("std".to_owned()))),
                            generics: vec![],
                        })),
                        generics: vec![],
                    }),
                    args,
                }))
            }

            Expression::Call {
                recipient: None,
                ref function,
                args,
            } if !args.is_empty() && is_bail(function) => {
                if let Some(error_type) = &ctx.error_type {
                    Expression::Call {
                        recipient: None,
                        function: Box::new(Expression::Ident("Err".to_owned())),
                        args: vec![Expression::Call {
                            recipient: None,
                            function: Box::new(Expression::Ident(format!("new {error_type}"))),
                            args,
                        }],
                    }
                } else {
                    Expression::Call {
                        recipient: None,
                        function: function.clone(),
                        args,
                    }
                }
            }

            other => other,
        }
    }
}

fn is_serde_json_from_str(function: &Expression) -> bool {
    matches!(
        function,
        Expression::Path { ident, parent: Some(parent), .. }
            if ident == "from_str"
                && matches!(
                    parent.as_ref(),
                    Expression::Ident(name) | Expression::Path { ident: name, .. }
                        if name == "serde_json"
                )
    )
}

fn is_std_thread_sleep(function: &Expression) -> bool {
    matches!(
        function,
        Expression::Path { ident, parent: Some(parent), .. }
            if ident == "sleep"
                && is_std_thread(parent)
    )
}

fn is_std_thread(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::Path { ident, parent: Some(parent), .. }
            if ident == "thread"
                && matches!(parent.as_ref(), Expression::Ident(name) if name == "std")
    )
}

fn is_bail(function: &Expression) -> bool {
    matches!(function, Expression::Ident(name) if name == "bail")
        || matches!(function, Expression::Path { ident, parent: None, .. } if ident == "bail")
}

#[cfg(test)]
mod tests {
    use crate::transpiler::rewrite::RewritePipeline;
    use super::*;

    #[test]
    fn serde_json_from_str_becomes_json_parse() {
        let mut expr = Expression::Call {
            recipient: None,
            function: Box::new(Expression::Path {
                ident: "from_str".to_owned(),
                parent: Some(Box::new(Expression::Ident("serde_json".to_owned()))),
                generics: vec![],
            }),
            args: vec![Expression::Ident("input".to_owned())],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsApi)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        match &expr {
            Expression::Call {
                recipient: None,
                function,
                args,
            } => {
                match function.as_ref() {
                    Expression::Path { ident, parent, .. } => {
                        assert_eq!(ident, "parse");
                        assert!(
                            matches!(parent.as_deref(), Some(Expression::Ident(s)) if s == "JSON")
                        );
                    }
                    _ => panic!("Expected Path, got {:?}", function),
                }
                assert_eq!(args.len(), 1);
                assert!(matches!(&args[0], Expression::Ident(s) if s == "input"));
            }
            _ => panic!("Expected Call, got {:?}", expr),
        }
    }

    #[test]
    fn std_thread_sleep_becomes_await() {
        let mut expr = Expression::Call {
            recipient: None,
            function: Box::new(Expression::Path {
                ident: "sleep".to_owned(),
                parent: Some(Box::new(Expression::Path {
                    ident: "thread".to_owned(),
                    parent: Some(Box::new(Expression::Ident("std".to_owned()))),
                    generics: vec![],
                })),
                generics: vec![],
            }),
            args: vec![Expression::Ident("duration".to_owned())],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsApi)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Await(_)));
    }

    #[test]
    fn bail_with_error_type_becomes_return_err() {
        let mut expr = Expression::Call {
            recipient: None,
            function: Box::new(Expression::Ident("bail".to_owned())),
            args: vec![Expression::Ident("msg".to_owned())],
        };

        let ctx = RewriteContext {
            error_type: Some("ApiError".to_owned()),
            ..Default::default()
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsApi)]);
        pipeline.rewrite_expr(&mut expr, &ctx);

        assert!(matches!(expr, Expression::Call { .. }));
        if let Expression::Call { function, args, .. } = &expr {
            assert!(matches!(function.as_ref(), Expression::Ident(s) if s == "Err"));
            assert_eq!(args.len(), 1);
        }
    }

    #[test]
    fn bail_without_error_type_unchanged() {
        let mut expr = Expression::Call {
            recipient: None,
            function: Box::new(Expression::Ident("bail".to_owned())),
            args: vec![Expression::Ident("msg".to_owned())],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsApi)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        assert!(matches!(expr, Expression::Call { .. }));
    }

    #[test]
    fn unrelated_call_unchanged() {
        let mut expr = Expression::Call {
            recipient: None,
            function: Box::new(Expression::Ident("println".to_owned())),
            args: vec![Expression::Ident("x".to_owned())],
        };

        let pipeline = RewritePipeline::new(vec![Box::new(RewriteTsApi)]);
        pipeline.rewrite_expr(&mut expr, &RewriteContext::default());

        match &expr {
            Expression::Call { function, .. } => {
                assert!(matches!(function.as_ref(), Expression::Ident(s) if s == "println"));
            }
            _ => panic!("Expected Call"),
        }
    }
}
