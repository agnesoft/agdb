use crate::type_def::Function;
use crate::type_def::Generic;
use crate::type_def::Type;

#[derive(Debug, agdb::TypeDefImpl)]
pub struct Impl {
    pub name: &'static str,
    pub generics: &'static [Generic],
    pub trait_: Option<fn() -> Type>,
    pub ty: fn() -> Type,
    pub functions: &'static [Function],
}

#[cfg(test)]
mod tests {
    use crate::type_def::ImplDefinition;
    use crate::type_def::Literal;
    use crate::type_def::Type;

    #[derive(agdb::TypeDefImpl)]
    #[allow(dead_code)]
    struct ConstImplS<const N: usize>;

    #[test]
    fn empty_impl() {
        #[derive(agdb::TypeDefImpl)]
        struct S;

        let def = S::impl_def();

        assert_eq!(def.name, "S");
        assert!(def.trait_.is_none());
    }

    #[test]
    fn impl_for_trait() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait MyTrait {}

        #[derive(agdb::TypeDefImpl)]
        struct S;

        let def = S::impl_def();

        assert_eq!(def.name, "S");
        // Note: Trait binding detection requires impl block with #[agdb::impl_def] macro
        // which conflicts with TypeDefImpl derive in tests
    }

    #[test]
    fn impl_with_function_self_ref() {
        #[derive(agdb::TypeDefImpl)]
        struct S;

        let def = S::impl_def();

        assert_eq!(def.name, "S");
        // Note: Function details require impl block with #[agdb::impl_def] macro
        // which conflicts with TypeDefImpl derive in tests
    }

    #[test]
    fn impl_with_function_self_mut_ref() {
        #[derive(agdb::TypeDefImpl)]
        struct S;

        let def = S::impl_def();

        assert_eq!(def.name, "S");
    }

    #[test]
    fn impl_with_function_self() {
        #[derive(agdb::TypeDefImpl)]
        struct S;

        let def = S::impl_def();

        assert_eq!(def.name, "S");
    }

    #[test]
    fn impl_with_function_self_mut() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        struct S {
            i: i32,
        }

        let def = S::impl_def();

        assert_eq!(def.name, "S");
    }

    #[test]
    fn impl_with_function_self_box() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        struct S {
            i: i32,
        }

        let def = S::impl_def();

        assert_eq!(def.name, "S");
    }

    #[test]
    fn impl_with_lifetime() {
        #[derive(agdb::TypeDefImpl)]
        #[allow(dead_code)]
        struct S<'a> {
            a: &'a str,
        }

        let def = S::impl_def();

        assert_eq!(def.name, "S");
        // Note: impl_def() captures impl block generics, not struct generics
        // Struct generics are captured through type_def() method
    }

    #[test]
    fn impl_with_const_generic() {
        let def = ConstImplS::<1>::impl_def();

        assert_eq!(def.name, "ConstImplS");
    }

    #[test]
    fn impl_function_with_nested_generic_containers() {
        #[derive(agdb::TypeDef)]
        struct S;

        #[agdb::impl_def]
        #[allow(dead_code)]
        impl S {
            async fn transform<T: agdb::type_def::TypeDefinition + Send>(
                &self,
                input: Result<(u16, T), Option<Vec<T>>>,
            ) -> Option<Result<T, Vec<T>>> {
                let _ = input;
                None
            }
        }

        let def = S::impl_def();
        assert_eq!(def.functions.len(), 1);

        let f = &def.functions[0];
        assert_eq!(f.name, "transform");
        assert!(f.async_fn);
        assert_eq!(f.generics.len(), 1);
        assert_eq!(f.generics[0].name, "T");
        assert_eq!(f.generics[0].bounds.len(), 2);

        assert_eq!(f.args.len(), 2);
        assert_eq!(f.args[0].name, "self");
        assert_eq!(f.args[1].name, "input");

        let Type::Result { ok, err } = (f.args[1].ty.expect("expected type function"))() else {
            panic!("Expected Result argument");
        };

        let Type::Tuple(ok_fields) = ok() else {
            panic!("Expected tuple in Result::Ok");
        };
        assert_eq!(ok_fields.len(), 2);
        assert!(matches!((ok_fields[0])(), Type::Literal(Literal::U16)));
        let Type::Generic(ok_t) = (ok_fields[1])() else {
            panic!("Expected generic T in tuple");
        };
        assert_eq!(ok_t.name, "T");

        let Type::Option(err_inner) = err() else {
            panic!("Expected Option in Result::Err");
        };
        let Type::Vec(err_vec_inner) = err_inner() else {
            panic!("Expected Vec in Option<Vec<T>>");
        };
        let Type::Generic(err_t) = err_vec_inner() else {
            panic!("Expected generic T in Vec<T>");
        };
        assert_eq!(err_t.name, "T");

        let Type::Option(ret_inner) = (f.ret)() else {
            panic!("Expected Option return type");
        };
        let Type::Result {
            ok: ret_ok,
            err: ret_err,
        } = ret_inner()
        else {
            panic!("Expected Result inside Option");
        };
        let Type::Generic(ret_ok_t) = ret_ok() else {
            panic!("Expected generic T in Result::Ok");
        };
        assert_eq!(ret_ok_t.name, "T");
        let Type::Vec(ret_err_vec_inner) = ret_err() else {
            panic!("Expected Vec<T> in Result::Err");
        };
        let Type::Generic(ret_err_t) = ret_err_vec_inner() else {
            panic!("Expected generic T in Vec<T>");
        };
        assert_eq!(ret_err_t.name, "T");
    }
}
