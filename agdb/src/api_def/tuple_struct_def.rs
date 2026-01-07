use crate::api_def::Function;
use crate::api_def::Generic;
use crate::api_def::Type;

#[derive(Debug)]
pub struct TupleStruct {
    pub name: &'static str,
    pub generics: &'static [Generic],
    pub fields: &'static [fn() -> Type],
    pub functions: &'static [Function],
}

#[cfg(test)]
mod tests {
    use crate::api_def::Type;
    use crate::api_def::TypeDefinition;

    #[test]
    fn tuple_definition() {
        #[derive(agdb::TypeDefImpl)]
        struct SomeTuple();

        let tuple_def = SomeTuple::type_def();

        if let Type::TupleStruct(t) = tuple_def {
            assert_eq!(t.name, "SomeTuple");
            assert_eq!(t.generics.len(), 0);
            assert_eq!(t.fields.len(), 0);
            assert_eq!(t.functions.len(), 0);
        } else {
            panic!("Expected Type::Tuple");
        }
    }

    #[test]
    fn tuple_definition_with_generics() {
        #[derive(agdb::TypeDefImpl)]
        struct GenericTuple<T>(T);

        let tuple_def = GenericTuple::<i32>::type_def();

        if let Type::TupleStruct(t) = tuple_def {
            assert_eq!(t.generics.len(), 1);
            assert_eq!(t.generics[0].name, "T");
            assert_eq!(t.generics[0].bounds.len(), 0);
            assert_eq!(t.fields.len(), 1);
            if let Type::Struct(s) = (t.fields[0])() {
                assert_eq!(s.name, "T");
            } else {
                panic!("Expected field type to be T Struct");
            }
        } else {
            panic!("Expected Type::Tuple");
        }
    }

    #[test]
    fn tuple_definition_with_concrete_type() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        struct ConcreteTuple(i32);

        let tuple_def = ConcreteTuple::type_def();

        if let Type::TupleStruct(t) = tuple_def {
            assert_eq!(t.fields.len(), 1);
            if let Type::Struct(s) = (t.fields[0])() {
                assert_eq!(s.name, "i32");
            } else {
                panic!("Expected field type to be T Struct");
            }
        } else {
            panic!("Expected Type::Tuple");
        }
    }

    #[test]
    fn tuple_definition_with_multiple_fields() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        struct ConcreteTuple(i32, String);

        let tuple_def = ConcreteTuple::type_def();

        if let Type::TupleStruct(t) = tuple_def {
            assert_eq!(t.fields.len(), 2);
            if let Type::Struct(s) = (t.fields[0])() {
                assert_eq!(s.name, "i32");
            } else {
                panic!("Expected field type to be T Struct");
            }
            if let Type::Struct(s) = (t.fields[1])() {
                assert_eq!(s.name, "String");
            } else {
                panic!("Expected field type to be T Struct");
            }
        } else {
            panic!("Expected Type::Tuple");
        }
    }
}
