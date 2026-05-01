use crate::type_def::Function;
use crate::type_def::Generic;
use crate::type_def::Type;

#[derive(Debug, agdb::TypeDef)]
pub struct Impl {
    pub name: &'static str,
    pub generics: &'static [Generic],
    pub trait_: Option<fn() -> Type>,
    pub ty: fn() -> Type,
    pub functions: &'static [Function],
}

#[cfg(test)]
mod tests {
    use crate::type_def::Literal;
    use crate::type_def::Type;
    use crate::type_def::TypeDefinition;

    #[derive(agdb::TypeDef)]
    #[allow(dead_code)]
    struct ConstImplS<const N: usize>;

    #[test]
    fn empty_impl() {
        #[derive(agdb::TypeDef)]
        struct S;

        assert!(S::impl_defs().is_empty());
    }

    #[test]
    fn impl_for_trait() {
        #[agdb::trait_def]
        #[allow(dead_code)]
        trait MyTrait {}

        #[derive(agdb::TypeDef)]
        struct S;

        assert!(S::impl_defs().is_empty());
    }

    #[test]
    fn impl_with_const_generic() {
        assert!(ConstImplS::<1>::impl_defs().is_empty());
    }

    #[test]
    fn impl_function_with_nested_generic_containers() {
        #[derive(agdb::TypeDef)]
        #[type_def(inherent)]
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

        let defs = S::impl_defs();
        assert_eq!(defs.len(), 1);
        let def = &defs[0];
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

    #[test]
    fn multiple_impl_blocks_with_drop() {
        #[derive(agdb::TypeDef)]
        #[type_def(inherent, Drop)]
        struct S;

        #[agdb::impl_def]
        #[allow(dead_code)]
        impl S {
            fn method(&self) -> i32 {
                42
            }
        }

        #[agdb::impl_def]
        impl Drop for S {
            fn drop(&mut self) {}
        }

        let defs = S::impl_defs();
        assert_eq!(defs.len(), 2);

        // Inherent impl: no trait
        assert!(defs[0].trait_.is_none());
        assert_eq!(defs[0].functions.len(), 1);
        assert_eq!(defs[0].functions[0].name, "method");

        // Drop impl: trait is Drop
        let trait_type = defs[1].trait_.expect("expected trait on Drop impl")();
        assert!(
            matches!(&trait_type, crate::type_def::Type::Trait(t) if t.name == "Drop"),
            "Got: {:?}",
            trait_type
        );
        assert_eq!(defs[1].functions.len(), 1);
        assert_eq!(defs[1].functions[0].name, "drop");
    }

    #[test]
    fn inherent_only() {
        #[derive(agdb::TypeDef)]
        #[type_def(inherent)]
        struct S;

        #[agdb::impl_def]
        #[allow(dead_code)]
        impl S {
            fn helper(&self) {}
        }

        let defs = S::impl_defs();
        assert_eq!(defs.len(), 1);
        assert!(defs[0].trait_.is_none());
        assert_eq!(defs[0].functions[0].name, "helper");
    }
}
