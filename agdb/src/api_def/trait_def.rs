use crate::api_def::Function;
use crate::api_def::Generic;
use crate::api_def::Type;

pub struct Trait {
    pub name: &'static str,
    pub generics: &'static [Generic],
    pub bounds: &'static [fn() -> Trait],
    pub types: &'static [fn() -> Type],
    pub functions: &'static [Function],
}

#[cfg(test)]
mod tests {
    use crate::api_def::Type;

    #[test]
    fn trait_definition() {
        #[agdb::trait_def()]
        #[allow(dead_code)]
        trait SomeTrait {}

        let trait_def = __SomeTrait_trait_def();

        assert_eq!(trait_def.name, "SomeTrait");
        assert_eq!(trait_def.generics.len(), 0);
        assert_eq!(trait_def.bounds.len(), 0);
        assert_eq!(trait_def.functions.len(), 0);
    }

    #[test]
    fn trait_definition_with_bounds() {
        #[agdb::trait_def()]
        #[allow(dead_code)]
        trait Bound1 {}
        #[agdb::trait_def()]
        #[allow(dead_code)]
        trait Bound2 {}
        #[agdb::trait_def()]
        #[allow(dead_code)]
        trait TraitWithBounds: Bound1 + Bound2 {}

        let trait_def = __TraitWithBounds_trait_def();

        assert_eq!(trait_def.bounds.len(), 2);
        assert_eq!((trait_def.bounds[0])().name, "Bound1");
        assert_eq!((trait_def.bounds[1])().name, "Bound2");
    }

    #[test]
    fn trait_definition_with_generics() {
        #[agdb::trait_def()]
        #[allow(dead_code)]
        trait Bound3 {}
        #[agdb::trait_def()]
        #[allow(dead_code)]
        trait GenericTrait<T: Bound3, U> {}

        let trait_def = __GenericTrait_trait_def();

        assert_eq!(trait_def.generics.len(), 2);
        assert_eq!(trait_def.generics[0].name, "T");
        assert_eq!(trait_def.generics[0].bounds.len(), 1);
        assert_eq!((trait_def.generics[0].bounds[0])().name, "Bound3");
        assert_eq!(trait_def.generics[1].name, "U");
        assert_eq!(trait_def.generics[1].bounds.len(), 0);
    }

    #[test]
    fn trait_definition_with_where() {
        #[agdb::trait_def()]
        #[allow(dead_code)]
        trait Bound3 {}
        #[agdb::trait_def()]
        #[allow(dead_code)]
        trait GenericTrait<T, U>
        where
            T: Bound3,
        {
        }

        let trait_def = __GenericTrait_trait_def();

        assert_eq!(trait_def.generics.len(), 2);
        assert_eq!(trait_def.generics[0].name, "T");
        assert_eq!(trait_def.generics[0].bounds.len(), 1);
        assert_eq!((trait_def.generics[0].bounds[0])().name, "Bound3");
        assert_eq!(trait_def.generics[1].name, "U");
        assert_eq!(trait_def.generics[1].bounds.len(), 0);
    }

    #[test]
    fn trait_definition_with_where_mixed() {
        #[agdb::trait_def()]
        #[allow(dead_code)]
        trait Bound3 {}
        #[agdb::trait_def()]
        #[allow(dead_code)]
        trait Bound4 {}
        #[agdb::trait_def()]
        #[allow(dead_code)]
        trait GenericTrait<T, U: Bound4>
        where
            T: Bound3,
        {
        }

        let trait_def = __GenericTrait_trait_def();

        assert_eq!(trait_def.generics.len(), 2);
        assert_eq!(trait_def.generics[0].name, "T");
        assert_eq!(trait_def.generics[0].bounds.len(), 1);
        assert_eq!((trait_def.generics[0].bounds[0])().name, "Bound3");
        assert_eq!(trait_def.generics[1].name, "U");
        assert_eq!(trait_def.generics[1].bounds.len(), 1);
        assert_eq!((trait_def.generics[1].bounds[0])().name, "Bound4");
    }

    #[test]
    fn trait_definition_with_functions() {
        #[agdb::trait_def()]
        #[allow(dead_code)]
        trait TraitWithFunctions {
            fn func1();
            fn func2(x: i32) -> i64;
        }

        let trait_def = __TraitWithFunctions_trait_def();

        assert_eq!(trait_def.functions.len(), 2);
        assert_eq!(trait_def.functions[0].name, "func1");
        assert_eq!(trait_def.functions[1].name, "func2");
        assert_eq!(trait_def.functions[1].args.len(), 1);
        assert!(trait_def.functions[1].ret.is_some());
        let ty = (trait_def.functions[1].args[0].ty).unwrap()();
        if let Type::Struct(s) = ty {
            assert_eq!(s.name, "i32");
        } else {
            panic!(
                "expected Type::Struct(i32), got {:?}",
                std::mem::discriminant(&ty)
            );
        }
        let ret = (trait_def.functions[1].ret.unwrap())();
        if let Type::Struct(s) = ret {
            assert_eq!(s.name, "i64");
        } else {
            panic!(
                "expected Type::Struct(i64), got {:?}",
                std::mem::discriminant(&ret)
            );
        }
    }

    #[test]
    fn trait_definition_with_types() {
        #[agdb::trait_def()]
        #[allow(dead_code)]
        trait TraitWithTypes {
            type AssociatedType1;
            type AssociatedType2;
        }

        let trait_def = __TraitWithTypes_trait_def();

        assert_eq!(trait_def.types.len(), 2);
        let ty1 = (trait_def.types[0])();
        if let Type::Struct(s) = ty1 {
            assert_eq!(s.name, "AssociatedType1");
        } else {
            panic!(
                "expected Type::Struct(AssociatedType1), got {:?}",
                std::mem::discriminant(&ty1)
            );
        }
        let ty2 = (trait_def.types[1])();
        if let Type::Struct(s) = ty2 {
            assert_eq!(s.name, "AssociatedType2");
        } else {
            panic!(
                "expected Type::Struct(AssociatedType2), got {:?}",
                std::mem::discriminant(&ty2)
            );
        }
    }
}
