#![cfg(feature = "api")]

use agdb::type_def::Expression;
use agdb::type_def::GenericKind;
use agdb::type_def::Literal;
use agdb::type_def::LiteralValue;
use agdb::type_def::Op;
use agdb::type_def::PointerKind;
use agdb::type_def::Type;
use agdb::type_def::TypeDefinition;

fn named_type(ty: &agdb::type_def::Variable) -> Type {
    (ty.ty.expect("expected type function"))()
}

fn get_body(name: &str) -> &'static [Expression] {
    let func_type = match name {
        "literal_integer" => __literal_integer_type_def(),
        "literal_bool" => __literal_bool_type_def(),
        "literal_string" => __literal_string_type_def(),
        "literal_float" => __literal_float_type_def(),
        "literal_suffixed" => __literal_suffixed_type_def(),
        "array_expr" => __array_expr_type_def(),
        "assign_expr" => __assign_expr_type_def(),
        "binary_arithmetic" => __binary_arithmetic_type_def(),
        "binary_comparison" => __binary_comparison_type_def(),
        "binary_logical" => __binary_logical_type_def(),
        "binary_assign_ops" => __binary_assign_ops_type_def(),
        "unary_neg" => __unary_neg_type_def(),
        "unary_not" => __unary_not_type_def(),
        "block_expr" => __block_expr_type_def(),
        "break_continue" => __break_continue_type_def(),
        "call_function" => __call_function_type_def(),
        "call_method" => __call_method_type_def(),
        "closure_simple" => __closure_simple_type_def(),
        "closure_typed" => __closure_typed_type_def(),
        "field_access_expr" => __field_access_expr_type_def(),
        "tuple_access_expr" => __tuple_access_expr_type_def(),
        "for_loop_expr" => __for_loop_expr_type_def(),
        "while_loop_expr" => __while_loop_expr_type_def(),
        "loop_expr" => __loop_expr_type_def(),
        "if_expr" => __if_expr_type_def(),
        "if_else_expr" => __if_else_expr_type_def(),
        "if_else_if_expr" => __if_else_if_expr_type_def(),
        "index_expr" => __index_expr_type_def(),
        "let_simple" => __let_simple_type_def(),
        "let_typed" => __let_typed_type_def(),
        "let_no_init" => __let_no_init_type_def(),
        "reference_expr" => __reference_expr_type_def(),
        "return_value" => __return_value_type_def(),
        "return_none" => __return_none_type_def(),
        "try_expr" => __try_expr_type_def(),
        "tuple_expr" => __tuple_expr_type_def(),
        "format_expr" => __format_expr_type_def(),
        "vec_macro_expr" => __vec_macro_expr_type_def(),
        "match_expr" => __match_expr_type_def(),
        "struct_expr" => __struct_expr_type_def(),
        "implicit_return" => __implicit_return_type_def(),
        "ident_expr" => __ident_expr_type_def(),
        "wild_expr" => __wild_expr_type_def(),
        "path_expr" => __path_expr_type_def(),
        "let_pattern_tuple" => __let_pattern_tuple_type_def(),
        "for_pattern" => __for_pattern_type_def(),
        _ => panic!("Unknown test function: {name}"),
    };

    let Type::Function(def) = func_type else {
        panic!("Expected function type definition");
    };

    def.body
}

#[test]
fn test_type_def_literals() {
    assert!(matches!(bool::type_def(), Type::Literal(Literal::Bool)));
    assert!(matches!(i32::type_def(), Type::Literal(Literal::I32)));
    assert!(matches!(f64::type_def(), Type::Literal(Literal::F64)));
    assert!(matches!(String::type_def(), Type::Literal(Literal::String)));
}

#[test]
fn options() {
    let Type::Option(inner) = Option::<i32>::type_def() else {
        panic!("Expected an option type definition");
    };

    assert!(matches!(inner(), Type::Literal(Literal::I32)));
}

#[test]
fn results() {
    let Type::Result { ok, err } = Result::<i32, String>::type_def() else {
        panic!("Expected a result type definition");
    };

    assert!(matches!(ok(), Type::Literal(Literal::I32)));
    assert!(matches!(err(), Type::Literal(Literal::String)));
}

#[test]
fn derive_type_enum_itself() {
    let Type::Enum(def) = Type::type_def() else {
        panic!("Expected enum type definition for Type");
    };

    assert_eq!(def.name, "Type");
    assert!(def.variants.iter().any(|v| v.name == "Function"));
    assert!(def.variants.iter().any(|v| v.name == "Trait"));
}

#[test]
fn function_pointer_type_def() {
    let Type::Function(def) = <fn(i32) -> String as TypeDefinition>::type_def() else {
        panic!("Expected function type definition");
    };

    assert_eq!(def.args.len(), 1);
    assert_eq!(def.args[0].name, "");
    assert!(matches!(
        named_type(&def.args[0]),
        Type::Literal(Literal::I32)
    ));
    assert!(matches!((def.ret)(), Type::Literal(Literal::String)));
}

#[test]
fn empty_struct() {
    #[derive(agdb::TypeDef)]
    struct S {}

    let Type::Struct(s) = S::type_def() else {
        panic!("Expected a struct type definition");
    };

    assert_eq!(s.name, "S");
}

#[test]
fn empty_struct_no_braces() {
    #[derive(agdb::TypeDef)]
    struct S;

    let Type::Struct(s) = S::type_def() else {
        panic!("Expected a struct type definition");
    };

    assert_eq!(s.name, "S");
}

#[test]
fn struct_with_fields() {
    #[derive(agdb::TypeDef)]
    struct S {
        _a: i32,
        _b: String,
    }

    let Type::Struct(s) = S::type_def() else {
        panic!("Expected a struct type definition");
    };

    assert_eq!(s.fields.len(), 2);
    assert_eq!(s.fields[0].name, "_a");
    assert_eq!(s.fields[1].name, "_b");
}

#[test]
fn tuple_struct_with_fields() {
    #[derive(agdb::TypeDef)]
    #[expect(dead_code)]
    struct S(i32, String);

    let Type::Struct(s) = S::type_def() else {
        panic!("Expected a struct type definition");
    };

    assert_eq!(s.fields.len(), 2);
    assert_eq!(s.fields[0].name, "");
    assert!(matches!(
        named_type(&s.fields[0]),
        Type::Literal(Literal::I32)
    ));
    assert_eq!(s.fields[1].name, "");
    assert!(matches!(
        named_type(&s.fields[1]),
        Type::Literal(Literal::String)
    ));
}

#[test]
fn field_types() {
    #[derive(agdb::TypeDef)]
    struct S {
        _a: i32,
    }

    let Type::Struct(s) = S::type_def() else {
        panic!("Expected a struct type definition");
    };

    assert!(matches!(
        named_type(&s.fields[0]),
        Type::Literal(Literal::I32)
    ));
}

#[test]
fn generic_struct() {
    #[derive(agdb::TypeDef)]
    struct S<T> {
        _a: T,
    }

    let Type::Struct(s) = S::<i32>::type_def() else {
        panic!("Expected a struct type definition");
    };

    let Type::Generic(generic) = named_type(&s.fields[0]) else {
        panic!("Expected a generic type definition");
    };

    assert_eq!(generic.name, "T");
    assert_eq!(generic.bounds.len(), 0);
}

#[test]
fn generic_struct_with_bounds() {
    #[agdb::trait_def]
    trait MyTrait {}

    impl MyTrait for i32 {}

    #[derive(agdb::TypeDef)]
    struct S<T: agdb::type_def::TypeDefinition + MyTrait> {
        _a: T,
    }

    let Type::Struct(s) = S::<i32>::type_def() else {
        panic!("Expected a struct type definition");
    };

    let Type::Generic(generic) = named_type(&s.fields[0]) else {
        panic!("Expected a generic type definition");
    };

    assert_eq!(generic.name, "T");
    assert_eq!(generic.bounds.len(), 2);
    assert!(matches!((generic.bounds[0])(), Type::Trait(_)));
    assert!(matches!((generic.bounds[1])(), Type::Trait(_)));
}

#[test]
fn generic_struct_with_where_clause() {
    #[derive(agdb::TypeDef)]
    struct S<T>
    where
        T: agdb::type_def::TypeDefinition,
    {
        _a: T,
    }

    let Type::Struct(s) = S::<i32>::type_def() else {
        panic!("Expected a struct type definition");
    };

    let Type::Generic(generic) = named_type(&s.fields[0]) else {
        panic!("Expected a generic type definition");
    };

    assert_eq!(generic.name, "T");
    assert_eq!(generic.bounds.len(), 1);
    assert!(matches!((generic.bounds[0])(), Type::Trait(_)));
}

#[test]
fn generic_tuple_struct() {
    #[derive(agdb::TypeDef)]
    struct S<T>(T);

    let Type::Struct(s) = S::<i32>::type_def() else {
        panic!("Expected a struct type definition");
    };

    let Type::Generic(generic) = named_type(&s.fields[0]) else {
        panic!("Expected a generic type definition");
    };

    assert_eq!(generic.name, "T");
    assert_eq!(generic.bounds.len(), 0);
}

#[test]
fn struct_with_lifetime() {
    #[derive(agdb::TypeDef)]
    struct S<'a> {
        _a: &'a str,
    }

    let Type::Struct(s) = S::type_def() else {
        panic!("Expected struct type definition");
    };

    assert_eq!(s.generics.len(), 1);
    assert!(matches!(s.generics[0].kind, GenericKind::Lifetime));
    assert_eq!(s.generics[0].name, "a");
}

#[test]
fn struct_with_const_generic() {
    #[derive(agdb::TypeDef)]
    struct S<const N: usize>;

    let Type::Struct(s) = S::<1>::type_def() else {
        panic!("Expected struct type definition");
    };

    assert_eq!(s.generics.len(), 1);
    assert!(matches!(s.generics[0].kind, GenericKind::Const));
    assert_eq!(s.generics[0].name, "N");
    assert_eq!(s.generics[0].bounds.len(), 1);
    assert!(matches!(
        (s.generics[0].bounds[0])(),
        Type::Literal(Literal::Usize)
    ));
}

#[test]
fn empty_enum() {
    #[derive(agdb::TypeDef)]
    enum E {}

    let Type::Enum(e) = E::type_def() else {
        panic!("Expected an enum type definition");
    };

    assert_eq!(e.name, "E");
}

#[test]
fn enum_with_variants() {
    #[derive(agdb::TypeDef)]
    #[expect(dead_code)]
    enum E {
        A,
        B,
    }

    let Type::Enum(e) = E::type_def() else {
        panic!("Expected an enum type definition");
    };

    assert_eq!(e.variants.len(), 2);
    assert_eq!(e.variants[0].name, "A");
    assert!(matches!(
        named_type(&e.variants[0]),
        Type::Literal(Literal::Unit)
    ));
    assert_eq!(e.variants[1].name, "B");
    assert!(matches!(
        named_type(&e.variants[1]),
        Type::Literal(Literal::Unit)
    ));
}

#[test]
fn enum_with_typed_variant() {
    #[derive(agdb::TypeDef)]
    #[expect(dead_code)]
    enum E {
        A(String),
    }

    let Type::Enum(e) = E::type_def() else {
        panic!("Expected an enum type definition");
    };

    assert!(matches!(
        named_type(&e.variants[0]),
        Type::Literal(Literal::String)
    ));
}

#[test]
fn enum_with_struct_variant() {
    #[derive(agdb::TypeDef)]
    #[expect(dead_code)]
    enum E {
        A { _a: String },
    }

    let Type::Enum(e) = E::type_def() else {
        panic!("Expected an enum type definition");
    };

    let Type::Struct(s) = named_type(&e.variants[0]) else {
        panic!("Expected a struct type definition");
    };
    assert_eq!(s.fields.len(), 1);
    assert_eq!(s.fields[0].name, "_a");
    assert!(matches!(
        named_type(&s.fields[0]),
        Type::Literal(Literal::String)
    ));
}

#[test]
fn enum_with_tuple_variant() {
    #[derive(agdb::TypeDef)]
    #[expect(dead_code)]
    enum E {
        A(String, i32),
    }

    let Type::Enum(e) = E::type_def() else {
        panic!("Expected an enum type definition");
    };

    let Type::Tuple(fields) = named_type(&e.variants[0]) else {
        panic!("Expected a tuple type definition");
    };
    assert_eq!(fields.len(), 2);
    assert!(matches!((fields[0])(), Type::Literal(Literal::String)));
    assert!(matches!((fields[1])(), Type::Literal(Literal::I32)));
}

#[test]
fn generic_enum() {
    #[derive(agdb::TypeDef)]
    #[expect(dead_code)]
    enum E<T> {
        A(T),
    }

    let Type::Enum(e) = E::<i32>::type_def() else {
        panic!("Expected an enum type definition");
    };

    let Type::Generic(generic) = named_type(&e.variants[0]) else {
        panic!("Expected a generic type definition");
    };

    assert_eq!(generic.name, "T");
    assert_eq!(generic.bounds.len(), 0);
}

#[test]
fn generic_enum_with_bounds() {
    #[agdb::trait_def]
    trait MyTrait {}

    impl MyTrait for i32 {}

    #[derive(agdb::TypeDef)]
    #[expect(dead_code)]
    enum E<T: agdb::type_def::TypeDefinition + MyTrait> {
        A(T),
    }

    let Type::Enum(e) = E::<i32>::type_def() else {
        panic!("Expected an enum type definition");
    };

    assert_eq!(e.generics[0].bounds.len(), 2);
    let Type::Generic(generic) = named_type(&e.variants[0]) else {
        panic!("Expected a generic type definition");
    };
    assert_eq!(generic.bounds.len(), 2);
}

#[test]
fn generic_enum_with_where_clause() {
    #[derive(agdb::TypeDef)]
    #[expect(dead_code)]
    enum E<T>
    where
        T: agdb::type_def::TypeDefinition,
    {
        A(T),
    }

    let Type::Enum(e) = E::<i32>::type_def() else {
        panic!("Expected an enum type definition");
    };

    let Type::Generic(generic) = named_type(&e.variants[0]) else {
        panic!("Expected a generic type definition");
    };
    assert_eq!(generic.bounds.len(), 1);
}

#[test]
fn generic_enum_with_struct_variant() {
    #[derive(agdb::TypeDef)]
    #[expect(dead_code)]
    enum E<T> {
        A { _a: T },
    }

    let Type::Enum(e) = E::<i32>::type_def() else {
        panic!("Expected an enum type definition");
    };

    let Type::Struct(s) = named_type(&e.variants[0]) else {
        panic!("Expected a struct type definition");
    };
    let Type::Generic(generic) = named_type(&s.fields[0]) else {
        panic!("Expected a generic type definition");
    };
    assert_eq!(generic.name, "T");
}

#[test]
fn generic_enum_with_tuple_variant_multiple_types() {
    #[derive(agdb::TypeDef)]
    #[expect(dead_code)]
    enum E<T> {
        A(T, i32),
    }

    let Type::Enum(e) = E::<String>::type_def() else {
        panic!("Expected an enum type definition");
    };

    let Type::Tuple(fields) = named_type(&e.variants[0]) else {
        panic!("Expected a tuple type definition");
    };
    let Type::Generic(generic) = (fields[0])() else {
        panic!("Expected a generic type definition");
    };
    assert_eq!(generic.name, "T");
    assert!(matches!((fields[1])(), Type::Literal(Literal::I32)));
}

#[test]
fn enum_with_lifetime() {
    #[derive(agdb::TypeDef)]
    #[expect(dead_code)]
    enum E<'a> {
        A(&'a str),
    }

    let Type::Enum(e) = E::type_def() else {
        panic!("Expected enum type definition");
    };

    assert!(matches!(e.generics[0].kind, GenericKind::Lifetime));
    assert_eq!(e.generics[0].name, "a");
}

#[test]
fn enum_with_const_generic() {
    #[derive(agdb::TypeDef)]
    #[expect(dead_code)]
    enum E<const N: usize> {
        A,
    }

    let Type::Enum(e) = E::<1>::type_def() else {
        panic!("Expected enum type definition");
    };

    assert!(matches!(e.generics[0].kind, GenericKind::Const));
    assert_eq!(e.generics[0].name, "N");
    assert!(matches!(
        (e.generics[0].bounds[0])(),
        Type::Literal(Literal::Usize)
    ));
}

#[test]
fn empty_function() {
    #[agdb::fn_def]
    #[expect(dead_code)]
    fn my_function() {}

    let Type::Function(def) = __my_function_type_def() else {
        panic!("Expected a function type definition");
    };

    assert_eq!(def.name, "my_function");
    assert_eq!(def.args.len(), 0);
    assert!(matches!((def.ret)(), Type::Literal(Literal::Unit)));
}

#[test]
fn function_with_arguments() {
    #[agdb::fn_def]
    #[allow(dead_code, unused_variables)]
    fn my_function(a: i32, b: String) {}

    let Type::Function(def) = __my_function_type_def() else {
        panic!("Expected a function type definition");
    };

    assert_eq!(def.args.len(), 2);
    assert_eq!(def.args[0].name, "a");
    assert_eq!(def.args[1].name, "b");
    assert!(matches!(
        named_type(&def.args[0]),
        Type::Literal(Literal::I32)
    ));
    assert!(matches!(
        named_type(&def.args[1]),
        Type::Literal(Literal::String)
    ));
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

    assert_eq!(def.generics.len(), 1);
    assert_eq!(def.generics[0].name, "T");
    assert_eq!(def.generics[0].bounds.len(), 1);
    let Type::Generic(arg_generic) = named_type(&def.args[0]) else {
        panic!("Expected a generic type definition");
    };
    assert_eq!(arg_generic.name, "T");
    let Type::Generic(ret_generic) = (def.ret)() else {
        panic!("Expected a generic return type definition");
    };
    assert_eq!(ret_generic.name, "T");
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

    assert!(matches!(def.generics[0].kind, GenericKind::Lifetime));
    assert_eq!(def.generics[0].name, "a");
    assert!(matches!(named_type(&def.args[0]), Type::Reference(_)));
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

    assert!(matches!(def.generics[0].kind, GenericKind::Lifetime));
    let Type::Reference(reference) = named_type(&def.args[0]) else {
        panic!("Expected reference type definition");
    };
    assert!(reference.mutable);
    assert_eq!(reference.lifetime, Some("a"));
}

#[test]
fn function_with_const_generic() {
    #[agdb::fn_def]
    #[expect(dead_code)]
    fn with_const<const N: usize>() {}

    let Type::Function(def) = __with_const_type_def() else {
        panic!("Expected function type definition");
    };

    assert!(matches!(def.generics[0].kind, GenericKind::Const));
    assert_eq!(def.generics[0].name, "N");
    assert!(matches!(
        (def.generics[0].bounds[0])(),
        Type::Literal(Literal::Usize)
    ));
}

#[test]
fn empty_impl() {
    #[derive(agdb::TypeDef)]
    #[type_def(inherent)]
    struct S;

    #[agdb::impl_def]
    impl S {}

    let defs = S::impl_defs();
    let def = &defs[0];

    assert_eq!(def.name, "S");
    assert!(def.trait_.is_none());
}

#[test]
fn impl_for_trait() {
    #[agdb::trait_def]
    #[allow(dead_code)]
    trait MyTrait {}

    #[derive(agdb::TypeDef)]
    #[type_def(MyTrait)]
    struct S;

    #[agdb::impl_def]
    impl MyTrait for S {}

    let defs = S::impl_defs();
    let def = &defs[0];
    let Some(tr) = def.trait_ else {
        panic!("Expected a trait bound");
    };
    let Type::Trait(trait_def) = tr() else {
        panic!("Expected a trait type definition");
    };
    assert_eq!(trait_def.name, "MyTrait");
}

#[test]
fn impl_with_function_self_ref() {
    #[derive(agdb::TypeDef)]
    #[type_def(inherent)]
    struct S;

    #[agdb::impl_def]
    #[expect(dead_code)]
    impl S {
        fn foo(&self) -> i32 {
            42
        }
    }

    let defs = S::impl_defs();
    let def = &defs[0];
    assert_eq!(def.functions[0].name, "foo");
    assert!(matches!(
        named_type(&def.functions[0].args[0]),
        Type::Reference(_)
    ));
}

#[test]
fn impl_with_function_self_mut_ref() {
    #[derive(agdb::TypeDef)]
    #[type_def(inherent)]
    struct S;

    #[agdb::impl_def]
    #[expect(dead_code)]
    impl S {
        fn foo(&mut self) -> i32 {
            42
        }
    }

    let defs = S::impl_defs();
    let def = &defs[0];
    let Type::Reference(reference) = named_type(&def.functions[0].args[0]) else {
        panic!("Expected reference type definition");
    };
    assert!(reference.mutable);
}

#[test]
fn impl_with_function_self() {
    #[derive(agdb::TypeDef)]
    #[type_def(inherent)]
    struct S;

    #[agdb::impl_def]
    #[expect(dead_code)]
    impl S {
        fn foo(self) -> i32 {
            42
        }
    }

    let defs = S::impl_defs();
    let def = &defs[0];
    assert!(matches!(
        named_type(&def.functions[0].args[0]),
        Type::SelfType(false)
    ));
}

#[test]
fn impl_with_function_self_mut() {
    #[derive(agdb::TypeDef)]
    #[type_def(inherent)]
    struct S {
        i: i32,
    }

    #[agdb::impl_def]
    #[expect(dead_code)]
    impl S {
        fn foo(mut self) -> i32 {
            self.i = 42;
            self.i
        }
    }

    let defs = S::impl_defs();
    let def = &defs[0];
    assert!(matches!(
        named_type(&def.functions[0].args[0]),
        Type::SelfType(true)
    ));
}

#[test]
fn impl_with_function_self_box() {
    #[derive(agdb::TypeDef)]
    #[type_def(inherent)]
    #[expect(dead_code)]
    struct S {
        i: i32,
    }

    #[agdb::impl_def]
    #[allow(clippy::boxed_local, dead_code)]
    impl S {
        fn foo(self: Box<Self>) -> i32 {
            42
        }
    }

    let defs = S::impl_defs();
    let def = &defs[0];
    let Type::Pointer(pointer) = named_type(&def.functions[0].args[0]) else {
        panic!("Expected pointer type definition");
    };
    assert!(matches!(pointer.kind, PointerKind::Box));
}

#[test]
fn impl_with_lifetime() {
    #[derive(agdb::TypeDef)]
    #[type_def(inherent)]
    struct S<'a> {
        a: &'a str,
    }

    #[agdb::impl_def]
    #[expect(dead_code)]
    impl<'a> S<'a> {
        fn get(&'a self) -> &'a str {
            self.a
        }
    }

    let defs = S::impl_defs();
    let def = &defs[0];
    assert!(matches!(def.generics[0].kind, GenericKind::Lifetime));
    let Type::Reference(reference) = named_type(&def.functions[0].args[0]) else {
        panic!("Expected reference type definition");
    };
    assert_eq!(reference.lifetime, Some("a"));
    assert!(!reference.mutable);
}

#[test]
fn impl_with_const_generic() {
    #[derive(agdb::TypeDef)]
    #[type_def(inherent)]
    struct ConstImplS<const N: usize>;

    #[agdb::impl_def]
    impl<const N: usize> ConstImplS<N> {}

    let defs = ConstImplS::<1>::impl_defs();
    let def = &defs[0];
    assert!(matches!(def.generics[0].kind, GenericKind::Const));
    assert_eq!(def.generics[0].name, "N");
}

#[test]
fn empty_trait() {
    #[agdb::trait_def]
    #[expect(dead_code)]
    trait MyTrait {}

    let Type::Trait(def) = MyTraitDef::type_def() else {
        panic!("Expected a trait type definition");
    };

    assert_eq!(def.name, "MyTrait");
}

#[test]
fn trait_with_generics() {
    #[agdb::trait_def]
    #[expect(dead_code)]
    trait MyTrait<T: agdb::type_def::TypeDefinition> {}

    let Type::Trait(def) = MyTraitDef::type_def() else {
        panic!("Expected a trait type definition");
    };

    assert_eq!(def.generics.len(), 1);
    assert_eq!(def.generics[0].name, "T");
    assert_eq!(def.generics[0].bounds.len(), 1);
}

#[test]
fn trait_with_where_clause() {
    #[agdb::trait_def]
    #[expect(dead_code)]
    trait MyTrait<T>
    where
        T: agdb::type_def::TypeDefinition,
    {
    }

    let Type::Trait(def) = MyTraitDef::type_def() else {
        panic!("Expected a trait type definition");
    };

    assert_eq!(def.generics.len(), 1);
    assert_eq!(def.generics[0].name, "T");
}

#[test]
fn trait_with_supertrait() {
    #[agdb::trait_def]
    #[expect(dead_code)]
    trait MyTrait: agdb::type_def::TypeDefinition {}

    let Type::Trait(def) = MyTraitDef::type_def() else {
        panic!("Expected a trait type definition");
    };

    assert_eq!(def.bounds.len(), 1);
}

#[test]
fn trait_with_functions() {
    #[agdb::trait_def]
    #[expect(dead_code)]
    trait MyTrait {
        fn a();
        async fn b(v: i32) -> String;
    }

    let Type::Trait(def) = MyTraitDef::type_def() else {
        panic!("Expected a trait type definition");
    };

    assert_eq!(def.functions.len(), 2);
    assert_eq!(def.functions[0].name, "a");
    assert!(!def.functions[0].async_fn);
    assert_eq!(def.functions[1].name, "b");
    assert!(def.functions[1].async_fn);
    assert!(matches!(
        named_type(&def.functions[1].args[0]),
        Type::Literal(Literal::I32)
    ));
}

#[test]
fn trait_function_with_generics() {
    #[agdb::trait_def]
    #[expect(dead_code)]
    trait MyTrait {
        fn id<T: agdb::type_def::TypeDefinition>(v: T) -> T;
    }

    let Type::Trait(def) = MyTraitDef::type_def() else {
        panic!("Expected a trait type definition");
    };

    let f = &def.functions[0];
    assert_eq!(f.name, "id");
    assert_eq!(f.generics.len(), 1);
    let Type::Generic(arg_generic) = named_type(&f.args[0]) else {
        panic!("Expected a generic type definition");
    };
    assert_eq!(arg_generic.name, "T");
}

#[test]
fn trait_with_lifetime() {
    #[agdb::trait_def]
    #[expect(dead_code)]
    trait MyLifetimeTrait<'a> {
        fn get(&'a self) -> &'a str;
    }

    let Type::Trait(def) = MyLifetimeTraitDef::type_def() else {
        panic!("Expected trait type definition");
    };

    assert!(matches!(def.generics[0].kind, GenericKind::Lifetime));
    let Type::Reference(reference) = named_type(&def.functions[0].args[0]) else {
        panic!("Expected reference type definition");
    };
    assert_eq!(reference.lifetime, Some("a"));
}

#[test]
fn trait_with_const_generic() {
    #[agdb::trait_def]
    #[expect(dead_code)]
    trait MyConstTrait<const N: usize> {}

    let Type::Trait(def) = MyConstTraitDef::type_def() else {
        panic!("Expected trait type definition");
    };

    assert!(matches!(def.generics[0].kind, GenericKind::Const));
    assert_eq!(def.generics[0].name, "N");
}

#[test]
fn trait_function_with_default_implementation() {
    #[agdb::trait_def]
    #[expect(dead_code)]
    trait MyTrait {
        fn with_default() {
            let _x = 42;
        }
    }

    let Type::Trait(def) = MyTraitDef::type_def() else {
        panic!("Expected a trait type definition");
    };

    let f = &def.functions[0];
    assert_eq!(f.name, "with_default");
    assert!(!f.body.is_empty());
}

#[test]
fn trait_function_without_default_implementation() {
    #[agdb::trait_def]
    #[expect(dead_code)]
    trait MyTrait {
        fn without_default();
    }

    let Type::Trait(def) = MyTraitDef::type_def() else {
        panic!("Expected a trait type definition");
    };

    assert!(def.functions[0].body.is_empty());
}

#[test]
fn trait_mixed_default_and_non_default_functions() {
    #[agdb::trait_def]
    #[expect(dead_code)]
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

    let Type::Trait(def) = MyTraitDef::type_def() else {
        panic!("Expected a trait type definition");
    };

    assert_eq!(def.functions.len(), 4);
    assert!(def.functions[0].body.is_empty());
    assert!(!def.functions[1].body.is_empty());
    assert!(def.functions[2].body.is_empty());
    assert!(!def.functions[3].body.is_empty());
}

#[test]
fn trait_default_function_with_generics() {
    #[agdb::trait_def]
    #[expect(dead_code)]
    trait MyTrait {
        fn apply<T: agdb::type_def::TypeDefinition>(val: T) -> T {
            val
        }
    }

    let Type::Trait(def) = MyTraitDef::type_def() else {
        panic!("Expected a trait type definition");
    };

    assert_eq!(def.functions[0].generics.len(), 1);
    assert!(!def.functions[0].body.is_empty());
}

#[test]
fn trait_default_async_function() {
    #[agdb::trait_def]
    #[expect(dead_code)]
    trait MyTrait {
        async fn async_with_default() {
            let _x = 1;
        }
    }

    let Type::Trait(def) = MyTraitDef::type_def() else {
        panic!("Expected a trait type definition");
    };

    assert!(def.functions[0].async_fn);
    assert!(!def.functions[0].body.is_empty());
}

#[agdb::fn_def]
#[allow(unused)]
fn literal_integer() -> i32 {
    42
}

#[test]
fn test_literal_integer() {
    let body = get_body("literal_integer");
    assert_eq!(body.len(), 1);
    assert!(matches!(
        &body[0],
        Expression::Return(Some(Expression::Literal(LiteralValue::I32(42))))
    ));
}

#[agdb::fn_def]
#[allow(unused)]
fn literal_bool() -> bool {
    true
}

#[test]
fn test_literal_bool() {
    let body = get_body("literal_bool");
    assert_eq!(body.len(), 1);
    assert!(matches!(
        &body[0],
        Expression::Return(Some(Expression::Literal(LiteralValue::Bool(true))))
    ));
}

#[agdb::fn_def]
#[allow(unused)]
fn literal_string() -> &'static str {
    "hello"
}

#[test]
fn test_literal_string() {
    let body = get_body("literal_string");
    match &body[0] {
        Expression::Return(Some(Expression::Literal(LiteralValue::Str(s)))) => {
            assert_eq!(*s, "hello");
        }
        _ => panic!("Got: {:?}", body[0]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn literal_float() -> f64 {
    3.3
}

#[test]
fn test_literal_float() {
    let body = get_body("literal_float");
    match &body[0] {
        Expression::Return(Some(Expression::Literal(LiteralValue::F64(v)))) => {
            assert!((*v - 3.3).abs() < f64::EPSILON);
        }
        _ => panic!("Got: {:?}", body[0]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn literal_suffixed() {
    let _a = 1u8;
    let _b = 2u16;
    let _c = 3u32;
    let _d = 4u64;
    let _e = 5i8;
    let _f = 6i16;
    let _g = 7i32;
    let _h = 8usize;
    let _i = 1.0f32;
}

#[test]
fn test_literal_suffixed() {
    let body = get_body("literal_suffixed");
    assert_eq!(body.len(), 9);
    assert!(matches!(
        &body[0],
        Expression::Let {
            value: Some(Expression::Literal(LiteralValue::U8(1))),
            ..
        }
    ));
    assert!(matches!(
        &body[1],
        Expression::Let {
            value: Some(Expression::Literal(LiteralValue::U16(2))),
            ..
        }
    ));
    assert!(matches!(
        &body[2],
        Expression::Let {
            value: Some(Expression::Literal(LiteralValue::U32(3))),
            ..
        }
    ));
    assert!(matches!(
        &body[3],
        Expression::Let {
            value: Some(Expression::Literal(LiteralValue::U64(4))),
            ..
        }
    ));
    assert!(matches!(
        &body[4],
        Expression::Let {
            value: Some(Expression::Literal(LiteralValue::I8(5))),
            ..
        }
    ));
    assert!(matches!(
        &body[5],
        Expression::Let {
            value: Some(Expression::Literal(LiteralValue::I16(6))),
            ..
        }
    ));
    assert!(matches!(
        &body[6],
        Expression::Let {
            value: Some(Expression::Literal(LiteralValue::I32(7))),
            ..
        }
    ));
    assert!(matches!(
        &body[7],
        Expression::Let {
            value: Some(Expression::Literal(LiteralValue::Usize(8))),
            ..
        }
    ));
    assert!(matches!(
        &body[8],
        Expression::Let {
            value: Some(Expression::Literal(LiteralValue::F32(_))),
            ..
        }
    ));
}

#[agdb::fn_def]
#[allow(unused)]
fn array_expr() {
    let _arr = [1, 2, 3];
}

#[test]
fn test_array() {
    let body = get_body("array_expr");
    match &body[0] {
        Expression::Let {
            value: Some(Expression::Array(elems)),
            ..
        } => assert_eq!(elems.len(), 3),
        _ => panic!("Got: {:?}", body[0]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn assign_expr() {
    let mut x = 1;
    x = 2;
}

#[test]
fn test_assign() {
    let body = get_body("assign_expr");
    match &body[1] {
        Expression::Assign { target, value } => {
            assert!(matches!(target, Expression::Ident("x")));
            assert!(matches!(value, Expression::Literal(LiteralValue::I32(2))));
        }
        _ => panic!("Got: {:?}", body[1]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn binary_arithmetic() {
    let _a = 1 + 2;
    let _b = 3 - 4;
    let _c = 5 * 6;
    let _d = 7 / 8;
    let _e = 19 % 10;
}

#[test]
fn test_binary_arithmetic() {
    let body = get_body("binary_arithmetic");
    assert!(matches!(
        &body[0],
        Expression::Let {
            value: Some(Expression::Binary { op: Op::Add, .. }),
            ..
        }
    ));
    assert!(matches!(
        &body[1],
        Expression::Let {
            value: Some(Expression::Binary { op: Op::Sub, .. }),
            ..
        }
    ));
    assert!(matches!(
        &body[2],
        Expression::Let {
            value: Some(Expression::Binary { op: Op::Mul, .. }),
            ..
        }
    ));
    assert!(matches!(
        &body[3],
        Expression::Let {
            value: Some(Expression::Binary { op: Op::Div, .. }),
            ..
        }
    ));
    assert!(matches!(
        &body[4],
        Expression::Let {
            value: Some(Expression::Binary { op: Op::Rem, .. }),
            ..
        }
    ));
}

#[agdb::fn_def]
#[allow(unused)]
fn binary_comparison() {
    let _a = 1 == 2;
    let _b = 1 != 2;
    let _c = 1 < 2;
    let _d = 1 <= 2;
    let _e = 1 > 2;
    let _f = 1 >= 2;
}

#[test]
fn test_binary_comparison() {
    let body = get_body("binary_comparison");
    let ops = [Op::Eq, Op::Ne, Op::Lt, Op::Le, Op::Gt, Op::Ge];
    for (i, expected_op) in ops.iter().enumerate() {
        match &body[i] {
            Expression::Let {
                value: Some(Expression::Binary { op, .. }),
                ..
            } => {
                assert!(std::mem::discriminant(op) == std::mem::discriminant(expected_op));
            }
            _ => panic!("body[{i}]: {:?}", body[i]),
        }
    }
}

#[agdb::fn_def]
#[allow(unused, clippy::nonminimal_bool)]
fn binary_logical() {
    let _a = true && false;
    let _b = true || false;
}

#[test]
fn test_binary_logical() {
    let body = get_body("binary_logical");
    assert!(matches!(
        &body[0],
        Expression::Let {
            value: Some(Expression::Binary { op: Op::And, .. }),
            ..
        }
    ));
    assert!(matches!(
        &body[1],
        Expression::Let {
            value: Some(Expression::Binary { op: Op::Or, .. }),
            ..
        }
    ));
}

#[agdb::fn_def]
#[allow(unused)]
fn binary_assign_ops() {
    let mut x = 0;
    x += 1;
    x -= 1;
    x *= 2;
    x /= 2;
    x %= 3;
}

#[test]
fn test_binary_assign_ops() {
    let body = get_body("binary_assign_ops");
    let ops = [
        Op::AddAssign,
        Op::SubAssign,
        Op::MulAssign,
        Op::DivAssign,
        Op::RemAssign,
    ];
    for (i, expected_op) in ops.iter().enumerate() {
        match &body[i + 1] {
            Expression::Binary { op, .. } => {
                assert!(std::mem::discriminant(op) == std::mem::discriminant(expected_op));
            }
            _ => panic!("body[{}]: {:?}", i + 1, body[i + 1]),
        }
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn unary_neg() -> i32 {
    -5
}

#[test]
fn test_unary_neg() {
    let body = get_body("unary_neg");
    assert!(matches!(
        &body[0],
        Expression::Return(Some(Expression::Unary { op: Op::Neg, .. }))
    ));
}

#[agdb::fn_def]
#[allow(unused, clippy::nonminimal_bool)]
fn unary_not() -> bool {
    !true
}

#[test]
fn test_unary_not() {
    let body = get_body("unary_not");
    assert!(matches!(
        &body[0],
        Expression::Return(Some(Expression::Unary { op: Op::Not, .. }))
    ));
}

#[agdb::fn_def]
#[allow(unused)]
fn block_expr() {
    {
        let _x = 1;
    };
}

#[test]
fn test_block() {
    let body = get_body("block_expr");
    match &body[0] {
        Expression::Block(stmts) => {
            assert_eq!(stmts.len(), 1);
            assert!(matches!(stmts[0], Expression::Let { .. }));
        }
        _ => panic!("Got: {:?}", body[0]),
    }
}

#[agdb::fn_def]
#[allow(unused, clippy::never_loop)]
fn break_continue() {
    loop {
        break;
    }
    loop {
        continue;
    }
}

#[test]
fn test_break_continue() {
    let body = get_body("break_continue");
    match &body[0] {
        Expression::While { body, .. } => match body {
            Expression::Block(stmts) => assert!(matches!(stmts[0], Expression::Break)),
            _ => panic!("Expected block, got {:?}", body),
        },
        _ => panic!("Got: {:?}", body[0]),
    }
    match &body[1] {
        Expression::While { body, .. } => match body {
            Expression::Block(stmts) => assert!(matches!(stmts[0], Expression::Continue)),
            _ => panic!("Expected block, got {:?}", body),
        },
        _ => panic!("Got: {:?}", body[1]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn call_function() {
    fn helper(_x: i32) -> i32 {
        0
    }
    let _r = helper(42);
}

#[test]
fn test_call_function() {
    let body = get_body("call_function");
    match &body[1] {
        Expression::Let {
            value:
                Some(Expression::Call {
                    recipient: None,
                    function,
                    args,
                }),
            ..
        } => {
            assert!(matches!(function, Expression::Ident("helper")));
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Got: {:?}", body[1]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn call_method() {
    let v = [1, 2, 3];
    let _len = v.len();
}

#[test]
fn test_call_method() {
    let body = get_body("call_method");
    match &body[1] {
        Expression::Let {
            value:
                Some(Expression::Call {
                    recipient: Some(_),
                    function,
                    args,
                }),
            ..
        } => {
            assert!(matches!(function, Expression::Path { ident: "len", .. }));
            assert_eq!(args.len(), 0);
        }
        _ => panic!("Got: {:?}", body[1]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn closure_simple() {
    let _f = |x: i32| x;
}

#[test]
fn test_closure_simple() {
    let body = get_body("closure_simple");
    match &body[0] {
        Expression::Let {
            value: Some(Expression::Closure(func)),
            ..
        } => {
            assert_eq!(func.name, "");
            assert_eq!(func.args.len(), 1);
            assert_eq!(func.args[0].name, "x");
        }
        _ => panic!("Got: {:?}", body[0]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn closure_typed() {
    let _f = |x: i32| -> i32 { x + 1 };
}

#[test]
fn test_closure_typed() {
    let body = get_body("closure_typed");
    match &body[0] {
        Expression::Let {
            value: Some(Expression::Closure(func)),
            ..
        } => {
            assert_eq!(func.args.len(), 1);
            assert_eq!(func.args[0].name, "x");
            assert!(!func.body.is_empty());
        }
        _ => panic!("Got: {:?}", body[0]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn field_access_expr() {
    struct S {
        field: i32,
    }
    let s = S { field: 1 };
    let _f = s.field;
}

#[test]
fn test_field_access() {
    let body = get_body("field_access_expr");
    match &body[2] {
        Expression::Let {
            value:
                Some(Expression::FieldAccess {
                    base,
                    field: "field",
                }),
            ..
        } => {
            assert!(matches!(base, Expression::Ident("s")));
        }
        _ => panic!("Got: {:?}", body[2]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn tuple_access_expr() {
    let t = (1, 2);
    let _first = t.0;
}

#[test]
fn test_tuple_access() {
    let body = get_body("tuple_access_expr");
    match &body[1] {
        Expression::Let {
            value: Some(Expression::TupleAccess { base, index: 0 }),
            ..
        } => {
            assert!(matches!(base, Expression::Ident("t")));
        }
        _ => panic!("Got: {:?}", body[1]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn for_loop_expr() {
    let items = [1, 2, 3];
    for _item in items {
        let _x = 1;
    }
}

#[test]
fn test_for_loop() {
    let body = get_body("for_loop_expr");
    match &body[1] {
        Expression::For {
            pattern,
            iterable,
            body,
        } => {
            assert!(matches!(pattern, Expression::Ident("_item")));
            assert!(matches!(iterable, Expression::Ident("items")));
            assert!(matches!(body, Expression::Block(_)));
        }
        _ => panic!("Got: {:?}", body[1]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn while_loop_expr() {
    let mut i = 0;
    while i < 10 {
        i += 1;
    }
}

#[test]
fn test_while_loop() {
    let body = get_body("while_loop_expr");
    match &body[1] {
        Expression::While { condition, body } => {
            assert!(matches!(condition, Expression::Binary { op: Op::Lt, .. }));
            assert!(matches!(body, Expression::Block(_)));
        }
        _ => panic!("Got: {:?}", body[1]),
    }
}

#[agdb::fn_def]
#[allow(unused, clippy::never_loop)]
fn loop_expr() {
    loop {
        break;
    }
}

#[test]
fn test_loop_desugars_to_while_true() {
    let body = get_body("loop_expr");
    match &body[0] {
        Expression::While { condition, .. } => {
            assert!(matches!(
                condition,
                Expression::Literal(LiteralValue::Bool(true))
            ));
        }
        _ => panic!("Got: {:?}", body[0]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn if_expr() {
    if true {
        let _x = 1;
    }
}

#[test]
fn test_if() {
    let body = get_body("if_expr");
    match &body[0] {
        Expression::If {
            condition,
            then_branch,
            else_branch: None,
        } => {
            assert!(matches!(
                condition,
                Expression::Literal(LiteralValue::Bool(true))
            ));
            assert!(matches!(then_branch, Expression::Block(_)));
        }
        _ => panic!("Got: {:?}", body[0]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn if_else_expr() {
    if true {
        let _x = 1;
    } else {
        let _y = 2;
    }
}

#[test]
fn test_if_else() {
    let body = get_body("if_else_expr");
    match &body[0] {
        Expression::If {
            else_branch: Some(eb),
            ..
        } => {
            assert!(matches!(eb, Expression::Block(_)));
        }
        _ => panic!("Got: {:?}", body[0]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn if_else_if_expr() {
    if true {
        let _x = 1;
    } else if false {
        let _y = 2;
    } else {
        let _z = 3;
    }
}

#[test]
fn test_if_else_if() {
    let body = get_body("if_else_if_expr");
    match &body[0] {
        Expression::If {
            else_branch:
                Some(Expression::If {
                    else_branch: Some(_),
                    ..
                }),
            ..
        } => {}
        _ => panic!("Got: {:?}", body[0]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn index_expr() {
    let arr = [1, 2, 3];
    let _v = arr[0];
}

#[test]
fn test_index() {
    let body = get_body("index_expr");
    match &body[1] {
        Expression::Let {
            value: Some(Expression::Index { base, index }),
            ..
        } => {
            assert!(matches!(base, Expression::Ident("arr")));
            assert!(matches!(index, Expression::Literal(LiteralValue::I32(0))));
        }
        _ => panic!("Got: {:?}", body[1]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn let_simple() {
    let _x = 42;
}

#[test]
fn test_let_simple() {
    let body = get_body("let_simple");
    match &body[0] {
        Expression::Let {
            name,
            ty: None,
            value: Some(Expression::Literal(LiteralValue::I32(42))),
        } => {
            assert!(matches!(name, Expression::Ident("_x")));
        }
        _ => panic!("Got: {:?}", body[0]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn let_typed() {
    let _x: i32 = 42;
}

#[test]
fn test_let_typed() {
    let body = get_body("let_typed");
    match &body[0] {
        Expression::Let {
            name,
            ty: Some(ty_fn),
            value: Some(_),
        } => {
            assert!(matches!(name, Expression::Ident("_x")));
            assert!(matches!(ty_fn(), Type::Literal(Literal::I32)));
        }
        _ => panic!("Got: {:?}", body[0]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn let_no_init() {
    let _x: i32;
}

#[test]
fn test_let_no_init() {
    let body = get_body("let_no_init");
    assert!(matches!(
        &body[0],
        Expression::Let {
            value: None,
            ty: Some(_),
            ..
        }
    ));
}

#[agdb::fn_def]
#[allow(unused)]
fn reference_expr() {
    let x = 42;
    let _r = &x;
}

#[test]
fn test_reference() {
    let body = get_body("reference_expr");
    match &body[1] {
        Expression::Let {
            value: Some(Expression::Reference(inner)),
            ..
        } => {
            assert!(matches!(inner, Expression::Ident("x")));
        }
        _ => panic!("Got: {:?}", body[1]),
    }
}

#[agdb::fn_def]
#[allow(unused, clippy::needless_return)]
fn return_value() -> i32 {
    return 42;
}

#[test]
fn test_return_value() {
    let body = get_body("return_value");
    assert!(matches!(
        &body[0],
        Expression::Return(Some(Expression::Literal(LiteralValue::I32(42))))
    ));
}

#[agdb::fn_def]
#[allow(unused, clippy::needless_return)]
fn return_none() {
    return;
}

#[test]
fn test_return_none() {
    let body = get_body("return_none");
    assert!(matches!(&body[0], Expression::Return(None)));
}

#[agdb::fn_def]
#[allow(unused)]
fn try_expr() -> Result<i32, String> {
    let r: Result<i32, String> = Ok(1);
    r?;
    Ok(0)
}

#[test]
fn test_try() {
    let body = get_body("try_expr");
    match &body[1] {
        Expression::Try(inner) => assert!(matches!(inner, Expression::Ident("r"))),
        _ => panic!("Got: {:?}", body[1]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn tuple_expr() {
    let _t = (1, 2, 3);
}

#[test]
fn test_tuple() {
    let body = get_body("tuple_expr");
    match &body[0] {
        Expression::Let {
            value: Some(Expression::Tuple(elems)),
            ..
        } => assert_eq!(elems.len(), 3),
        _ => panic!("Got: {:?}", body[0]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn format_expr() {
    let x = 42;
    let _s = format!("{}", x);
}

#[test]
fn test_format_macro() {
    let body = get_body("format_expr");
    match &body[1] {
        Expression::Let {
            value:
                Some(Expression::Format {
                    format_string,
                    args,
                }),
            ..
        } => {
            assert_eq!(*format_string, "{}");
            assert_eq!(args.len(), 1);
            assert!(matches!(args[0], Expression::Ident("x")));
        }
        _ => panic!("Got: {:?}", body[1]),
    }
}

#[agdb::fn_def]
#[allow(unused, clippy::useless_vec)]
fn vec_macro_expr() {
    let _v = vec![1, 2, 3];
}

#[test]
fn test_vec_macro() {
    let body = get_body("vec_macro_expr");
    match &body[0] {
        Expression::Let {
            value: Some(Expression::Array(elems)),
            ..
        } => assert_eq!(elems.len(), 3),
        _ => panic!("Got: {:?}", body[0]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn match_expr() -> i32 {
    let x = 1;
    match x {
        1 => 10,
        2 => 20,
        _ => 0,
    }
}

#[test]
fn test_match() {
    let body = get_body("match_expr");
    match &body[1] {
        Expression::If {
            condition,
            then_branch,
            else_branch: Some(_),
        } => {
            assert!(matches!(condition, Expression::Binary { op: Op::Eq, .. }));
            assert!(matches!(then_branch, Expression::Block(_)));
        }
        _ => panic!("Got: {:?}", body[1]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn struct_expr() {
    struct Point {
        x: i32,
        y: i32,
    }
    let _p = Point { x: 1, y: 2 };
}

#[test]
fn test_struct_expr() {
    let body = get_body("struct_expr");
    match &body[1] {
        Expression::Let {
            value: Some(Expression::Struct { name, fields }),
            ..
        } => {
            assert!(matches!(name, Expression::Ident("Point")));
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "x");
            assert_eq!(fields[1].0, "y");
        }
        _ => panic!("Got: {:?}", body[1]),
    }
}

#[agdb::fn_def]
#[allow(unused, clippy::let_and_return)]
fn implicit_return() -> i32 {
    let x = 5;
    x
}

#[test]
fn test_implicit_return() {
    let body = get_body("implicit_return");
    assert!(matches!(
        &body[1],
        Expression::Return(Some(Expression::Ident("x")))
    ));
}

#[agdb::fn_def]
#[allow(unused)]
fn ident_expr() {
    let x = 1;
    let _y = x;
}

#[test]
fn test_ident() {
    let body = get_body("ident_expr");
    assert!(matches!(
        &body[1],
        Expression::Let {
            value: Some(Expression::Ident("x")),
            ..
        }
    ));
}

#[agdb::fn_def]
#[allow(unused)]
fn wild_expr() {
    let _ = 42;
}

#[test]
fn test_wild() {
    let body = get_body("wild_expr");
    assert!(matches!(
        &body[0],
        Expression::Let {
            name: Expression::Wild,
            ..
        }
    ));
}

#[agdb::fn_def]
#[allow(unused)]
fn path_expr() {
    let _v: Option<i32> = None;
}

#[test]
fn test_path() {
    let body = get_body("path_expr");
    assert!(matches!(
        &body[0],
        Expression::Let {
            value: Some(Expression::Ident("None")),
            ..
        }
    ));
}

#[agdb::fn_def]
#[allow(unused)]
fn let_pattern_tuple() {
    let (a, b) = (1, 2);
    let _ = a + b;
}

#[test]
fn test_let_pattern_tuple() {
    let body = get_body("let_pattern_tuple");
    match &body[0] {
        Expression::Let {
            name: Expression::Tuple(elems),
            ..
        } => {
            assert_eq!(elems.len(), 2);
            assert!(matches!(elems[0], Expression::Ident("a")));
            assert!(matches!(elems[1], Expression::Ident("b")));
        }
        _ => panic!("Got: {:?}", body[0]),
    }
}

#[agdb::fn_def]
#[allow(unused)]
fn for_pattern() {
    let items = [(1, 2), (3, 4)];
    for (a, _b) in items {
        let _ = a;
    }
}

#[test]
fn test_for_pattern() {
    let body = get_body("for_pattern");
    match &body[1] {
        Expression::For { pattern, .. } => assert!(matches!(pattern, Expression::Tuple(_))),
        _ => panic!("Got: {:?}", body[1]),
    }
}
