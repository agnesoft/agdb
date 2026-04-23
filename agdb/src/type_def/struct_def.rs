use crate::type_def::Function;
use crate::type_def::Generic;
use crate::type_def::Variable;

#[derive(Debug, agdb::TypeDefImpl)]
pub struct Struct {
    pub name: &'static str,
    pub generics: &'static [Generic],
    pub fields: &'static [Variable],
    pub functions: &'static [Function],
}

#[cfg(test)]
mod tests {
    use crate::type_def::GenericKind;
    use crate::type_def::Literal;
    use crate::type_def::Type;
    use crate::type_def::TypeDefinition;

    #[test]
    fn empty_struct() {
        #[derive(agdb::TypeDefImpl)]
        struct S {}

        let Type::Struct(s) = S::type_def() else {
            panic!("Expected a struct type definition");
        };

        assert_eq!(s.name, "S");
    }

    #[test]
    fn empty_struct_no_braces() {
        #[derive(agdb::TypeDefImpl)]
        struct S;

        let Type::Struct(s) = S::type_def() else {
            panic!("Expected a struct type definition");
        };

        assert_eq!(s.name, "S");
    }

    #[test]
    fn struct_with_fields() {
        #[derive(agdb::TypeDefImpl)]
        struct S {
            _a: i32,
            _b: String,
        }

        let Type::Struct(s) = S::type_def() else {
            panic!("Expected a struct type definition");
        };

        assert_eq!(s.name, "S");
        assert_eq!(s.fields.len(), 2);
        assert_eq!(s.fields[0].name, "_a");
        assert_eq!(s.fields[1].name, "_b");
    }

    #[test]
    fn tuple_struct_with_fields() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        struct S(i32, String);

        let Type::Struct(s) = S::type_def() else {
            panic!("Expected a struct type definition");
        };

        assert_eq!(s.name, "S");
        assert_eq!(s.fields.len(), 2);
        assert_eq!(s.fields[0].name, "");

        let Type::Literal(Literal::I32) = (s.fields[0].ty.expect("expected type function"))() else {
            panic!("Expected a literal type definition");
        };

        assert_eq!(s.fields[1].name, "");

        let Type::Literal(Literal::String) = (s.fields[1].ty.expect("expected type function"))() else {
            panic!("Expected a literal type definition");
        };
    }

    #[test]
    fn field_types() {
        #[derive(agdb::TypeDefImpl)]
        struct S {
            _a: i32,
        }

        let Type::Struct(s) = S::type_def() else {
            panic!("Expected a struct type definition");
        };

        let ty = (s.fields[0].ty.expect("expected type function"))();

        let Type::Literal(Literal::I32) = ty else {
            panic!("Expected a literal type definition");
        };
    }

    #[test]
    fn generic_struct() {
        #[derive(agdb::TypeDefImpl)]
        struct S<T> {
            _a: T,
        }

        let Type::Struct(s) = S::<i32>::type_def() else {
            panic!("Expected a struct type definition");
        };

        assert_eq!(s.name, "S");
        assert_eq!(s.fields.len(), 1);
        assert_eq!(s.fields[0].name, "_a");

        let ty = (s.fields[0].ty.expect("expected type function"))();

        let Type::Generic(generic) = ty else {
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

        #[derive(agdb::TypeDefImpl)]
        struct S<T: agdb::type_def::TypeDefinition + MyTrait> {
            _a: T,
        }

        let Type::Struct(s) = S::<i32>::type_def() else {
            panic!("Expected a struct type definition");
        };

        assert_eq!(s.name, "S");
        assert_eq!(s.fields.len(), 1);
        assert_eq!(s.fields[0].name, "_a");

        let ty = (s.fields[0].ty.expect("expected type function"))();

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
    fn generic_struct_with_where_clause() {
        #[derive(agdb::TypeDefImpl)]
        struct S<T>
        where
            T: agdb::type_def::TypeDefinition,
        {
            _a: T,
        }

        let Type::Struct(s) = S::<i32>::type_def() else {
            panic!("Expected a struct type definition");
        };

        assert_eq!(s.name, "S");
        assert_eq!(s.fields.len(), 1);
        assert_eq!(s.fields[0].name, "_a");

        let ty = (s.fields[0].ty.expect("expected type function"))();

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
    fn generic_tuple_struct() {
        #[derive(agdb::TypeDefImpl)]
        struct S<T>(T);

        let Type::Struct(s) = S::<i32>::type_def() else {
            panic!("Expected a struct type definition");
        };

        assert_eq!(s.name, "S");
        assert_eq!(s.fields.len(), 1);
        assert_eq!(s.fields[0].name, "");

        let ty = (s.fields[0].ty.expect("expected type function"))();

        let Type::Generic(generic) = ty else {
            panic!("Expected a generic type definition");
        };

        assert_eq!(generic.name, "T");
        assert_eq!(generic.bounds.len(), 0);
    }

    #[test]
    fn struct_with_lifetime() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        struct S<'a> {
            _a: &'a str,
        }

        let Type::Struct(s) = S::type_def() else {
            panic!("Expected struct type definition");
        };

        assert_eq!(s.generics.len(), 1);
        assert!(matches!(s.generics[0].kind, GenericKind::Lifetime));
        assert_eq!(s.generics[0].name, "a");
        assert_eq!(s.generics[0].bounds.len(), 0);
        assert_eq!(s.name, "S");
    }

    #[test]
    fn struct_with_const_generic() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        struct S<const N: usize>;

        let Type::Struct(s) = S::<1>::type_def() else {
            panic!("Expected struct type definition");
        };

        assert_eq!(s.generics.len(), 1);
        assert!(matches!(s.generics[0].kind, GenericKind::Const));
        assert_eq!(s.generics[0].name, "N");
        assert_eq!(s.generics[0].bounds.len(), 1);
        assert!(
            matches!((s.generics[0].bounds[0])(), Type::Literal(Literal::Usize)),
            "Got: {:?}",
            (s.generics[0].bounds[0])()
        );
    }
}
