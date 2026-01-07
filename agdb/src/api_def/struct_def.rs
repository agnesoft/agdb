use crate::api_def::Function;
use crate::api_def::Generic;
use crate::api_def::NamedType;

#[derive(Debug)]
pub struct Struct {
    pub name: &'static str,
    pub generics: &'static [Generic],
    pub fields: &'static [NamedType],
    pub functions: &'static [Function],
}

#[cfg(test)]
mod tests {
    use crate::api_def::Type;
    use crate::api_def::TypeDefinition;

    #[test]
    fn struct_definition() {
        #[derive(agdb::TypeDefImpl)]
        struct SomeStruct;

        let struct_def = SomeStruct::type_def();

        if let Type::Struct(s) = struct_def {
            assert_eq!(s.name, "SomeStruct");
            assert_eq!(s.generics.len(), 0);
            assert_eq!(s.fields.len(), 0);
        } else {
            panic!("Expected Type::Struct");
        }
    }

    #[test]
    fn struct_definition_with_generics() {
        #[derive(agdb::TypeDefImpl)]
        struct GenericStruct<T> {
            #[allow(dead_code)]
            field: T,
        }

        let struct_def = GenericStruct::<i32>::type_def();

        if let Type::Struct(s) = struct_def {
            assert_eq!(s.generics.len(), 1);
            assert_eq!(s.generics[0].name, "T");
            assert_eq!(s.generics[0].bounds.len(), 0);
            assert_eq!(s.fields.len(), 1);
            assert_eq!(s.fields[0].name, "field");
        } else {
            panic!("Expected Type::Struct");
        }
    }

    #[test]
    fn struct_definition_with_generics_with_bounds() {
        trait Bound5 {}

        #[derive(agdb::TypeDefImpl)]
        struct GenericStruct<T>
        where
            T: Bound5,
        {
            #[allow(dead_code)]
            field: T,
        }

        struct BoundedStruct;
        impl Bound5 for BoundedStruct {}

        let struct_def = GenericStruct::<BoundedStruct>::type_def();

        if let Type::Struct(s) = struct_def {
            assert_eq!(s.generics.len(), 1);
            assert_eq!(s.generics[0].name, "T");
            assert_eq!(s.generics[0].bounds.len(), 1);
            assert_eq!(s.generics[0].bounds[0], "Bound5");
        } else {
            panic!("Expected Type::Struct");
        }
    }

    #[test]
    fn struct_definition_with_fields() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        struct FieldStruct {
            a: i32,
            b: String,
        }

        let struct_def = FieldStruct::type_def();

        if let Type::Struct(s) = struct_def {
            assert_eq!(s.fields.len(), 2);
            assert_eq!(s.fields[0].name, "a");
            assert_eq!(s.fields[1].name, "b");
        } else {
            panic!("Expected Type::Struct");
        }
    }
}
