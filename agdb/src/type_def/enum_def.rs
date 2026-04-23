use crate::type_def::Function;
use crate::type_def::Generic;
use crate::type_def::Variable;

#[derive(Debug, agdb::TypeDefImpl)]
pub struct Enum {
    pub name: &'static str,
    pub generics: &'static [Generic],
    pub variants: &'static [Variable],
    pub functions: &'static [Function],
}

#[cfg(test)]
mod tests {
    use crate::type_def::GenericKind;
    use crate::type_def::Literal;
    use crate::type_def::Type;
    use crate::type_def::TypeDefinition;

    #[test]
    fn empty_enum() {
        #[derive(agdb::TypeDefImpl)]
        enum E {}

        let Type::Enum(e) = E::type_def() else {
            panic!("Expected an enum type definition");
        };

        assert_eq!(e.name, "E");
    }

    #[test]
    fn enum_with_variants() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        enum E {
            A,
            B,
        }

        let Type::Enum(e) = E::type_def() else {
            panic!("Expected an enum type definition");
        };

        assert_eq!(e.name, "E");
        assert_eq!(e.variants.len(), 2);
        assert_eq!(e.variants[0].name, "A");
        assert!(
            matches!((e.variants[0].ty.expect("expected type function"))(), Type::Literal(Literal::Unit),),
            "Got: {:?}",
            (e.variants[0].ty.expect("expected type function"))()
        );
        assert_eq!(e.variants[1].name, "B");
        assert!(
            matches!((e.variants[1].ty.expect("expected type function"))(), Type::Literal(Literal::Unit),),
            "Got: {:?}",
            (e.variants[1].ty.expect("expected type function"))()
        );
    }

    #[test]
    fn enum_with_typed_variant() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        enum E {
            A(String),
        }

        let Type::Enum(e) = E::type_def() else {
            panic!("Expected an enum type definition");
        };

        assert_eq!(e.name, "E");
        assert_eq!(e.variants.len(), 1);
        assert_eq!(e.variants[0].name, "A");
        assert!(
            matches!((e.variants[0].ty.expect("expected type function"))(), Type::Literal(Literal::String)),
            "Got: {:?}",
            (e.variants[0].ty.expect("expected type function"))()
        );
    }

    #[test]
    fn enum_with_struct_variant() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        enum E {
            A { _a: String },
        }

        let Type::Enum(e) = E::type_def() else {
            panic!("Expected an enum type definition");
        };

        assert_eq!(e.name, "E");
        assert_eq!(e.variants.len(), 1);
        assert_eq!(e.variants[0].name, "A");
        let Type::Struct(s) = (e.variants[0].ty.expect("expected type function"))() else {
            panic!("Expected a struct type definition");
        };
        assert_eq!(s.fields.len(), 1);
        assert_eq!(s.fields[0].name, "_a");
        assert!(
            matches!((s.fields[0].ty.expect("expected type function"))(), Type::Literal(Literal::String)),
            "Got: {:?}",
            (s.fields[0].ty.expect("expected type function"))()
        );
    }

    #[test]
    fn enum_with_tuple_variant() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        enum E {
            A(String, i32),
        }

        let Type::Enum(e) = E::type_def() else {
            panic!("Expected an enum type definition");
        };

        assert_eq!(e.name, "E");
        assert_eq!(e.variants.len(), 1);
        assert_eq!(e.variants[0].name, "A");
        let Type::Tuple(fields) = (e.variants[0].ty.expect("expected type function"))() else {
            panic!("Expected a tuple type definition");
        };
        assert_eq!(fields.len(), 2);
        assert!(
            matches!((fields[0])(), Type::Literal(Literal::String)),
            "Got: {:?}",
            (fields[0])()
        );
        assert!(
            matches!((fields[1])(), Type::Literal(Literal::I32)),
            "Got: {:?}",
            (fields[1])()
        );
    }

    #[test]
    fn generic_enum() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        enum E<T> {
            A(T),
        }

        let Type::Enum(e) = E::<i32>::type_def() else {
            panic!("Expected an enum type definition");
        };

        assert_eq!(e.name, "E");
        assert_eq!(e.generics.len(), 1);
        assert_eq!(e.generics[0].name, "T");
        assert_eq!(e.generics[0].bounds.len(), 0);
        assert_eq!(e.variants.len(), 1);
        assert_eq!(e.variants[0].name, "A");

        let ty = (e.variants[0].ty.expect("expected type function"))();

        let Type::Generic(generic) = ty else {
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

        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        enum E<T: agdb::type_def::TypeDefinition + MyTrait> {
            A(T),
        }

        let Type::Enum(e) = E::<i32>::type_def() else {
            panic!("Expected an enum type definition");
        };

        assert_eq!(e.name, "E");
        assert_eq!(e.generics.len(), 1);
        assert_eq!(e.generics[0].name, "T");
        assert_eq!(e.generics[0].bounds.len(), 2);
        let Type::Trait(bound_trait) = (e.generics[0].bounds[0])() else {
            panic!("Expected a trait type definition");
        };

        assert_eq!(bound_trait.name, "TypeDefinition");

        let Type::Trait(bound_trait) = (e.generics[0].bounds[1])() else {
            panic!("Expected a trait type definition");
        };

        assert_eq!(bound_trait.name, "MyTrait");

        assert_eq!(e.variants.len(), 1);
        assert_eq!(e.variants[0].name, "A");
        let ty = (e.variants[0].ty.expect("expected type function"))();

        let Type::Generic(generic) = ty else {
            panic!("Expected a generic type definition");
        };

        assert_eq!(generic.name, "T");
        assert_eq!(generic.bounds.len(), 2);
        let Type::Trait(bound_trait) = (generic.bounds[0])() else {
            panic!("Expected a trait type definition");
        };

        assert_eq!(bound_trait.name, "TypeDefinition");

        let Type::Trait(bound_trait) = (generic.bounds[1])() else {
            panic!("Expected a trait type definition");
        };

        assert_eq!(bound_trait.name, "MyTrait");
    }

    #[test]
    fn generic_enum_with_where_clause() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        enum E<T>
        where
            T: agdb::type_def::TypeDefinition,
        {
            A(T),
        }

        let Type::Enum(e) = E::<i32>::type_def() else {
            panic!("Expected an enum type definition");
        };

        assert_eq!(e.name, "E");
        assert_eq!(e.generics.len(), 1);
        assert_eq!(e.generics[0].name, "T");
        assert_eq!(e.generics[0].bounds.len(), 1);
        let Type::Trait(bound_trait) = (e.generics[0].bounds[0])() else {
            panic!("Expected a trait type definition");
        };

        assert_eq!(bound_trait.name, "TypeDefinition");

        assert_eq!(e.variants.len(), 1);
        assert_eq!(e.variants[0].name, "A");
        let ty = (e.variants[0].ty.expect("expected type function"))();

        let Type::Generic(generic) = ty else {
            panic!("Expected a generic type definition");
        };

        assert_eq!(generic.name, "T");
        assert_eq!(generic.bounds.len(), 1);
        let Type::Trait(bound_trait) = (generic.bounds[0])() else {
            panic!("Expected a trait type definition");
        };

        assert_eq!(bound_trait.name, "TypeDefinition");
    }

    #[test]
    fn generic_enum_with_struct_variant() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        enum E<T> {
            A { _a: T },
        }

        let Type::Enum(e) = E::<i32>::type_def() else {
            panic!("Expected an enum type definition");
        };

        assert_eq!(e.name, "E");
        assert_eq!(e.generics.len(), 1);
        assert_eq!(e.generics[0].name, "T");
        assert_eq!(e.generics[0].bounds.len(), 0);
        assert_eq!(e.variants.len(), 1);
        assert_eq!(e.variants[0].name, "A");

        let Type::Struct(s) = (e.variants[0].ty.expect("expected type function"))() else {
            panic!("Expected a struct type definition");
        };
        assert_eq!(s.fields.len(), 1);
        assert_eq!(s.fields[0].name, "_a");

        let Type::Generic(generic) = (s.fields[0].ty.expect("expected type function"))() else {
            panic!("Expected a generic type definition");
        };

        assert_eq!(generic.name, "T");
        assert_eq!(generic.bounds.len(), 0);
    }

    #[test]
    fn generic_enum_with_tuple_variant_multiple_types() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        enum E<T> {
            A(T, i32),
        }

        let Type::Enum(e) = E::<String>::type_def() else {
            panic!("Expected an enum type definition");
        };

        assert_eq!(e.name, "E");
        assert_eq!(e.generics.len(), 1);
        assert_eq!(e.generics[0].name, "T");
        assert_eq!(e.generics[0].bounds.len(), 0);
        assert_eq!(e.variants.len(), 1);
        assert_eq!(e.variants[0].name, "A");

        let Type::Tuple(fields) = (e.variants[0].ty.expect("expected type function"))() else {
            panic!("Expected a tuple type definition");
        };

        assert_eq!(fields.len(), 2);

        let Type::Generic(generic) = (fields[0])() else {
            panic!("Expected a generic type definition");
        };

        assert_eq!(generic.name, "T");
        assert_eq!(generic.bounds.len(), 0);
        assert!(
            matches!((fields[1])(), Type::Literal(Literal::I32)),
            "Got: {:?}",
            (fields[1])()
        );
    }

    #[test]
    fn enum_with_lifetime() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        enum E<'a> {
            A(&'a str),
        }

        let Type::Enum(e) = E::type_def() else {
            panic!("Expected enum type definition");
        };

        assert_eq!(e.generics.len(), 1);
        assert!(matches!(e.generics[0].kind, GenericKind::Lifetime));
        assert_eq!(e.generics[0].name, "a");
        assert_eq!(e.generics[0].bounds.len(), 0);
        assert_eq!(e.name, "E");
    }

    #[test]
    fn enum_with_const_generic() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        enum E<const N: usize> {
            A,
        }

        let Type::Enum(e) = E::<1>::type_def() else {
            panic!("Expected enum type definition");
        };

        assert_eq!(e.generics.len(), 1);
        assert!(matches!(e.generics[0].kind, GenericKind::Const));
        assert_eq!(e.generics[0].name, "N");
        assert_eq!(e.generics[0].bounds.len(), 1);
        assert!(
            matches!((e.generics[0].bounds[0])(), Type::Literal(Literal::Usize)),
            "Got: {:?}",
            (e.generics[0].bounds[0])()
        );
        assert_eq!(e.name, "E");
    }
}
