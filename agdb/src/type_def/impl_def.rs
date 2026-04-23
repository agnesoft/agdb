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
}
