use crate::api_def::Function;
use crate::api_def::GenericParam;
use crate::api_def::NamedType;

#[derive(Debug)]
pub struct Enum {
    pub name: &'static str,
    pub generic_params: &'static [GenericParam],
    pub variants: &'static [NamedType],
    pub functions: &'static [Function],
}

#[cfg(test)]
mod tests {
    use crate::api_def::Type;
    use crate::api_def::TypeDefinition;

    #[test]
    fn enum_definition() {
        #[derive(agdb::TypeDefImpl)]
        enum SomeEnum {}

        let enum_def = SomeEnum::type_def();

        if let Type::Enum(e) = enum_def {
            assert_eq!(e.name, "SomeEnum");
            assert_eq!(e.generic_params.len(), 0);
            assert_eq!(e.variants.len(), 0);
        } else {
            panic!("Expected Type::Enum");
        }
    }

    #[test]
    fn enum_definition_with_variants() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        enum SimpleEnum {
            A,
            B,
            C,
        }

        let enum_def = SimpleEnum::type_def();

        if let Type::Enum(e) = enum_def {
            assert_eq!(e.variants.len(), 3);
            assert_eq!(e.variants[0].name, "A");
            assert_eq!(e.variants[1].name, "B");
            assert_eq!(e.variants[2].name, "C");
        } else {
            panic!("Expected Type::Enum");
        }
    }

    #[test]
    fn enum_definition_with_generics() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        enum GenericEnum<T, U> {
            Variant1 { field1: T },
            Variant2 { field2: U },
        }

        let enum_def = GenericEnum::<i32, f64>::type_def();

        if let Type::Enum(e) = enum_def {
            assert_eq!(e.generic_params.len(), 2);
            assert_eq!(e.generic_params[0].name, "T");
            assert_eq!(e.generic_params[1].name, "U");
            assert_eq!(e.variants.len(), 2);
            assert_eq!(e.variants[0].name, "Variant1");
            if let Type::Struct(s) = (e.variants[0].ty.unwrap())() {
                assert_eq!(s.fields.len(), 1);
                assert_eq!(s.fields[0].name, "field1");
            } else {
                panic!("Expected Variant1 field type to be Struct");
            }
            assert_eq!(e.variants[1].name, "Variant2");
            if let Type::Struct(s) = (e.variants[1].ty.unwrap())() {
                assert_eq!(s.fields.len(), 1);
                assert_eq!(s.fields[0].name, "field2");
            } else {
                panic!("Expected Variant2 field type to be Struct");
            }
        } else {
            panic!("Expected Type::Enum");
        }
    }

    #[test]
    fn enum_definition_variant_of_another_type() {
        #[derive(agdb::TypeDef)]
        struct OtherType;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl OtherType {
            fn foo() {}
        }

        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        enum OtherTypeEnum {
            TupleVariant(OtherType),
        }

        let enum_def = OtherTypeEnum::type_def();

        if let Type::Enum(e) = enum_def {
            assert_eq!(e.variants.len(), 1);
            assert_eq!(e.variants[0].name, "TupleVariant");

            println!("Variant type: {:?}", e.variants[0].ty.unwrap()());

            if let Type::Struct(s) = (e.variants[0].ty.unwrap())() {
                assert_eq!(s.name, "OtherType");
                s.functions
                    .iter()
                    .find(|f| f.name == "foo")
                    .expect("Expected function foo");
            } else {
                panic!("Expected UnitVariant type not to be None");
            }
        } else {
            panic!("Expected Type::Enum");
        }
    }
}
