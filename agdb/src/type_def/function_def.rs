use crate::type_def::Generic;
use crate::type_def::Type;
use crate::type_def::Variable;
use crate::type_def::expression_def::Expression;

#[derive(Debug, agdb::TypeDefImpl)]
pub struct Function {
    pub name: &'static str,
    pub generics: &'static [Generic],
    pub args: &'static [Variable],
    pub ret: fn() -> Type,
    pub async_fn: bool,
    pub body: &'static [Expression],
}

#[cfg(test)]
mod tests {
    use crate::type_def::Expression;
    use crate::type_def::GenericKind;
    use crate::type_def::Literal;
    use crate::type_def::Type;

    #[agdb::test_def]
    #[test]
    fn plain_test_function() {
        let value = 1;
        assert!(value == 1);
    }

    #[agdb::test_def]
    #[tokio::test]
    async fn tokio_async_test_function() {
        let value = 1;
        assert_eq!(value, 1);
        assert!(matches!(Some(value), Some(1)));
    }

    #[test]
    fn plain_test_type_def() {
        let Type::Test(def) = __plain_test_function_type_def() else {
            panic!("Expected a test type definition");
        };

        assert_eq!(def.name, "plain_test_function");
        assert!(!def.async_fn);
        assert_eq!(def.body.len(), 2);
        assert!(
            matches!(
                &def.body[1],
                Expression::Call {
                    function: Expression::Path {
                        ident: "assert",
                        ..
                    },
                    ..
                }
            ),
            "Got: {:?}",
            def.body[1]
        );
    }

    #[test]
    fn tokio_test_type_def() {
        let Type::Test(def) = __tokio_async_test_function_type_def() else {
            panic!("Expected a test type definition");
        };

        assert_eq!(def.name, "tokio_async_test_function");
        assert!(def.async_fn);
        assert_eq!(def.body.len(), 3);

        assert!(
            matches!(
                &def.body[1],
                Expression::Call {
                    function: Expression::Path {
                        ident: "assert_eq",
                        ..
                    },
                    ..
                }
            ),
            "Got: {:?}",
            def.body[1]
        );

        let Expression::Call { args, .. } = &def.body[2] else {
            panic!("Expected assert! call in body");
        };
        assert!(
            matches!(
                &args[0],
                Expression::Call {
                    function: Expression::Path {
                        ident: "matches",
                        ..
                    },
                    ..
                }
            ),
            "Got: {:?}",
            args[0]
        );
    }

    #[test]
    fn empty_function() {
        #[agdb::fn_def]
        #[allow(dead_code)]
        fn my_function() {}

        let Type::Function(def) = __my_function_type_def() else {
            panic!("Expected a function type definition");
        };

        assert_eq!(def.name, "my_function");
        assert_eq!(def.args.len(), 0);
        assert!(
            matches!((def.ret)(), Type::Literal(Literal::Unit),),
            "Got: {:?}",
            (def.ret)()
        );
    }

    #[test]
    fn function_with_arguments() {
        #[agdb::fn_def]
        #[allow(dead_code, unused_variables)]
        fn my_function(a: i32, b: String) {}

        let Type::Function(def) = __my_function_type_def() else {
            panic!("Expected a function type definition");
        };

        assert_eq!(def.name, "my_function");
        assert_eq!(def.args.len(), 2);
        assert_eq!(def.args[0].name, "a");
        assert_eq!(def.args[1].name, "b");
        assert!(
            matches!(
                (def.args[0].ty.expect("expected type function"))(),
                Type::Literal(Literal::I32),
            ),
            "Got: {:?}",
            (def.args[0].ty.expect("expected type function"))()
        );
        assert!(
            matches!(
                (def.args[1].ty.expect("expected type function"))(),
                Type::Literal(Literal::String),
            ),
            "Got: {:?}",
            (def.args[1].ty.expect("expected type function"))()
        );
    }

    #[test]
    fn generic_function_argument_and_return() {
        #[agdb::fn_def]
        #[allow(dead_code, unused_variables)]
        fn my_function<T: agdb::type_def::TypeDefinition>(value: T) -> T {
            panic!("body should not be used by fn_def parser")
        }

        let Type::Function(def) = __my_function_type_def() else {
            panic!("Expected a function type definition");
        };

        assert_eq!(def.name, "my_function");
        assert_eq!(def.generics.len(), 1);
        assert_eq!(def.generics[0].name, "T");
        assert_eq!(def.generics[0].bounds.len(), 1);

        let Type::Trait(bound_trait) = (def.generics[0].bounds[0])() else {
            panic!("Expected a trait type definition");
        };
        assert_eq!(bound_trait.name, "TypeDefinition");

        assert_eq!(def.args.len(), 1);
        assert_eq!(def.args[0].name, "value");

        let Type::Generic(arg_generic) = (def.args[0].ty.expect("expected type function"))() else {
            panic!("Expected a generic type definition");
        };
        assert_eq!(arg_generic.name, "T");
        assert_eq!(arg_generic.bounds.len(), 1);

        let Type::Generic(ret_generic) = (def.ret)() else {
            panic!("Expected a generic return type definition");
        };
        assert_eq!(ret_generic.name, "T");
        assert_eq!(ret_generic.bounds.len(), 1);
    }

    #[test]
    fn function_with_lifetime() {
        #[agdb::fn_def]
        #[allow(dead_code, clippy::needless_lifetimes)]
        fn borrow<'a>(s: &'a str) -> &'a str {
            s
        }

        let Type::Function(def) = __borrow_type_def() else {
            panic!("Expected function type definition");
        };

        assert_eq!(def.generics.len(), 1);
        assert!(matches!(def.generics[0].kind, GenericKind::Lifetime));
        assert_eq!(def.generics[0].name, "a");
        assert_eq!(def.generics[0].bounds.len(), 0);
        assert_eq!(def.name, "borrow");
        assert_eq!(def.args.len(), 1);
        assert_eq!(def.args[0].name, "s");
        assert!(
            matches!(
                (def.args[0].ty.expect("expected type function"))(),
                Type::Reference(crate::type_def::Reference {
                    mutable: false,
                    lifetime: Some("a"),
                    ty: _
                }),
            ),
            "Got: {:?}",
            (def.args[0].ty.expect("expected type function"))()
        );
    }

    #[test]
    fn function_with_args_with_lifetime() {
        #[agdb::fn_def]
        #[allow(dead_code, clippy::needless_lifetimes)]
        fn borrow<'a>(s: &'a mut Vec<String>) -> &'a Vec<String> {
            s
        }

        let Type::Function(def) = __borrow_type_def() else {
            panic!("Expected function type definition");
        };

        assert_eq!(def.generics.len(), 1);
        assert!(matches!(def.generics[0].kind, GenericKind::Lifetime));
        assert_eq!(def.generics[0].name, "a");
        assert_eq!(def.generics[0].bounds.len(), 0);
        assert_eq!(def.name, "borrow");
        assert_eq!(def.args.len(), 1);
        assert_eq!(def.args[0].name, "s");
        assert!(
            matches!(
                (def.args[0].ty.expect("expected type function"))(),
                Type::Reference(crate::type_def::Reference {
                    mutable: true,
                    lifetime: Some("a"),
                    ty: _
                }),
            ),
            "Got: {:?}",
            (def.args[0].ty.expect("expected type function"))()
        );
    }

    #[test]
    fn function_with_const_generic() {
        #[agdb::fn_def]
        #[allow(dead_code)]
        fn with_const<const N: usize>() {}

        let Type::Function(def) = __with_const_type_def() else {
            panic!("Expected function type definition");
        };

        assert_eq!(def.generics.len(), 1);
        assert!(matches!(def.generics[0].kind, GenericKind::Const));
        assert_eq!(def.generics[0].name, "N");
        assert_eq!(def.generics[0].bounds.len(), 1);
        assert!(
            matches!((def.generics[0].bounds[0])(), Type::Literal(Literal::Usize)),
            "Got: {:?}",
            (def.generics[0].bounds[0])()
        );
    }
}
