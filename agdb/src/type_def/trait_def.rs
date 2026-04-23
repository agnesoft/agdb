use crate::type_def::Function;
use crate::type_def::Generic;
use crate::type_def::Type;

#[derive(Debug, agdb::TypeDefImpl)]
pub struct Trait {
    pub name: &'static str,
    pub generics: &'static [Generic],
    pub bounds: &'static [fn() -> Type],
    pub functions: &'static [Function],
}

#[cfg(test)]
mod tests {
    use crate::type_def::GenericKind;
    use crate::type_def::Literal;
    use crate::type_def::Type;

    #[test]
    fn empty_trait() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait MyTrait {}

        let Type::Trait(def) = __MyTrait_type_def() else {
            panic!("Expected a trait type definition");
        };

        assert_eq!(def.name, "MyTrait");
    }

    #[test]
    fn trait_with_generics() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait MyTrait<T: agdb::type_def::TypeDefinition> {}

        let Type::Trait(def) = __MyTrait_type_def() else {
            panic!("Expected a trait type definition");
        };

        assert_eq!(def.generics.len(), 1);
        let generic = &def.generics[0];
        assert_eq!(generic.name, "T");
        assert_eq!(generic.bounds.len(), 1);
        let Type::Trait(bound_trait) = (generic.bounds[0])() else {
            panic!("Expected a trait type definition");
        };
        assert_eq!(bound_trait.name, "TypeDefinition");
    }

    #[test]
    fn trait_with_where_clause() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait MyTrait<T>
        where
            T: agdb::type_def::TypeDefinition,
        {
        }

        let Type::Trait(def) = __MyTrait_type_def() else {
            panic!("Expected a trait type definition");
        };

        assert_eq!(def.generics.len(), 1);
        let generic = &def.generics[0];
        assert_eq!(generic.name, "T");
        assert_eq!(generic.bounds.len(), 1);
        let Type::Trait(bound_trait) = (generic.bounds[0])() else {
            panic!("Expected a trait type definition");
        };
        assert_eq!(bound_trait.name, "TypeDefinition");
    }

    #[test]
    fn trait_with_supertrait() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait MyTrait: agdb::type_def::TypeDefinition {}

        let Type::Trait(def) = __MyTrait_type_def() else {
            panic!("Expected a trait type definition");
        };

        assert_eq!(def.bounds.len(), 1);
        let Type::Trait(bound_trait) = (def.bounds[0])() else {
            panic!("Expected a trait type definition");
        };
        assert_eq!(bound_trait.name, "TypeDefinition");
    }

    #[test]
    fn trait_with_functions() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait MyTrait {
            fn a();
            async fn b(v: i32) -> String;
        }

        let Type::Trait(def) = __MyTrait_type_def() else {
            panic!("Expected a trait type definition");
        };

        assert_eq!(def.functions.len(), 2);

        let a = &def.functions[0];
        assert_eq!(a.name, "a");
        assert_eq!(a.args.len(), 0);
        assert!(!a.async_fn);
        assert!(
            matches!((a.ret)(), Type::Literal(Literal::Unit),),
            "Got: {:?}",
            (a.ret)()
        );

        let b = &def.functions[1];
        assert_eq!(b.name, "b");
        assert_eq!(b.args.len(), 1);
        assert_eq!(b.args[0].name, "v");
        assert!(b.async_fn);
        assert!(
            matches!((b.args[0].ty.expect("expected type function"))(), Type::Literal(Literal::I32),),
            "Got: {:?}",
            (b.args[0].ty.expect("expected type function"))()
        );
        assert!(
            matches!((b.ret)(), Type::Literal(Literal::String),),
            "Got: {:?}",
            (b.ret)()
        );
    }

    #[test]
    fn trait_function_with_generics() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait MyTrait {
            fn id<T: agdb::type_def::TypeDefinition>(v: T) -> T;
        }

        let Type::Trait(def) = __MyTrait_type_def() else {
            panic!("Expected a trait type definition");
        };

        assert_eq!(def.functions.len(), 1);
        let f = &def.functions[0];
        assert_eq!(f.name, "id");
        assert_eq!(f.generics.len(), 1);
        assert_eq!(f.generics[0].name, "T");
        assert_eq!(f.generics[0].bounds.len(), 1);

        let Type::Trait(bound_trait) = (f.generics[0].bounds[0])() else {
            panic!("Expected a trait type definition");
        };
        assert_eq!(bound_trait.name, "TypeDefinition");

        let Type::Generic(arg_generic) = (f.args[0].ty.expect("expected type function"))() else {
            panic!("Expected a generic type definition");
        };
        assert_eq!(arg_generic.name, "T");

        let Type::Generic(ret_generic) = (f.ret)() else {
            panic!("Expected a generic return type definition");
        };
        assert_eq!(ret_generic.name, "T");
    }

    #[test]
    fn trait_with_lifetime() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait MyLifetimeTrait<'a> {
            fn get(&'a self) -> &'a str;
        }

        let Type::Trait(def) = __MyLifetimeTrait_type_def() else {
            panic!("Expected trait type definition");
        };

        assert_eq!(def.generics.len(), 1);
        assert!(matches!(def.generics[0].kind, GenericKind::Lifetime));
        assert_eq!(def.generics[0].name, "a");
        assert_eq!(def.generics[0].bounds.len(), 0);
        assert_eq!(def.functions.len(), 1);
        assert_eq!(def.functions[0].generics.len(), 0);
        assert_eq!(def.functions[0].args.len(), 1);
        assert_eq!(def.functions[0].args[0].name, "self");
        assert!(
            matches!(
                (def.functions[0].args[0].ty.expect("expected type function"))(),
                Type::Reference(crate::type_def::Reference {
                    mutable: false,
                    lifetime: Some("a"),
                    ty: _
                }),
            ),
            "Got: {:?}",
            (def.functions[0].args[0].ty.expect("expected type function"))()
        );
    }

    #[test]
    fn trait_with_const_generic() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait MyConstTrait<const N: usize> {}

        let Type::Trait(def) = __MyConstTrait_type_def() else {
            panic!("Expected trait type definition");
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

    #[test]
    fn trait_function_with_default_implementation() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait MyTrait {
            fn with_default() {
                let _x = 42;
            }
        }

        let Type::Trait(def) = __MyTrait_type_def() else {
            panic!("Expected a trait type definition");
        };

        assert_eq!(def.functions.len(), 1);
        let f = &def.functions[0];
        assert_eq!(f.name, "with_default");
        assert_eq!(f.args.len(), 0);
        assert!(!f.async_fn);
        assert!(!f.body.is_empty(), "Expected a body for default implementation");
        assert_eq!(f.body.len(), 1);
    }

    #[test]
    fn trait_function_without_default_implementation() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait MyTrait {
            fn without_default();
        }

        let Type::Trait(def) = __MyTrait_type_def() else {
            panic!("Expected a trait type definition");
        };

        assert_eq!(def.functions.len(), 1);
        let f = &def.functions[0];
        assert_eq!(f.name, "without_default");
        assert!(f.body.is_empty(), "Expected empty body for trait function without default implementation");
    }

    #[test]
    fn trait_mixed_default_and_non_default_functions() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait MyTrait {
            fn required();
            fn with_default() {
                let _x = 42;
            }
            fn another_required(a: i32);
            fn another_default(_b: String) -> bool {
                true
            }
        }

        let Type::Trait(def) = __MyTrait_type_def() else {
            panic!("Expected a trait type definition");
        };

        assert_eq!(def.functions.len(), 4);

        let required = &def.functions[0];
        assert_eq!(required.name, "required");
        assert!(required.body.is_empty());

        let with_default = &def.functions[1];
        assert_eq!(with_default.name, "with_default");
        assert!(!with_default.body.is_empty());

        let another_required = &def.functions[2];
        assert_eq!(another_required.name, "another_required");
        assert!(another_required.body.is_empty());
        assert_eq!(another_required.args.len(), 1);
        assert_eq!(another_required.args[0].name, "a");

        let another_default = &def.functions[3];
        assert_eq!(another_default.name, "another_default");
        assert!(!another_default.body.is_empty());
        assert_eq!(another_default.args.len(), 1);
        assert_eq!(another_default.args[0].name, "_b");
    }

    #[test]
    fn trait_default_function_with_generics() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait MyTrait {
            fn apply<T: agdb::type_def::TypeDefinition>(val: T) -> T {
                val
            }
        }

        let Type::Trait(def) = __MyTrait_type_def() else {
            panic!("Expected a trait type definition");
        };

        assert_eq!(def.functions.len(), 1);
        let f = &def.functions[0];
        assert_eq!(f.name, "apply");
        assert_eq!(f.generics.len(), 1);
        assert_eq!(f.generics[0].name, "T");
        assert_eq!(f.args.len(), 1);
        assert_eq!(f.args[0].name, "val");
        assert!(!f.body.is_empty(), "Expected body for default generic function");
    }

    #[test]
    fn trait_default_async_function() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait MyTrait {
            async fn async_with_default() {
                let _x = 1;
            }
        }

        let Type::Trait(def) = __MyTrait_type_def() else {
            panic!("Expected a trait type definition");
        };

        assert_eq!(def.functions.len(), 1);
        let f = &def.functions[0];
        assert_eq!(f.name, "async_with_default");
        assert!(f.async_fn);
        assert!(!f.body.is_empty());
    }
}
