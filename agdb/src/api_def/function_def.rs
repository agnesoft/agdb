use crate::api_def::GenericParam;
use crate::api_def::NamedType;
use crate::api_def::Type;
use crate::api_def::expression_def::Expression;

#[derive(Debug)]
pub struct Function {
    pub name: &'static str,
    pub generic_params: &'static [GenericParam],
    pub args: &'static [NamedType],
    pub ret: Option<fn() -> Type>,
    pub async_fn: bool,
    pub expressions: &'static [Expression],
}

#[cfg(test)]
mod tests {
    use crate::api_def::TypeDefinition;

    #[test]
    fn simple_function() {
        #[derive(agdb::TypeDef)]
        struct StructWithFunction;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithFunction {
            fn example_function() {}
        }

        let functions = StructWithFunction::type_def().functions();

        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "example_function");
        assert!(functions[0].generic_params.is_empty());
        assert!(functions[0].args.is_empty());
        assert!(functions[0].ret.is_none());
        assert!(functions[0].expressions.is_empty());
    }

    #[test]
    fn function_with_args_and_return() {
        #[derive(agdb::TypeDef)]
        struct StructWithFunctionArgs;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithFunctionArgs {
            fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        }

        let functions = StructWithFunctionArgs::type_def().functions();

        assert_eq!(functions.len(), 1);
        let func = &functions[0];
        assert_eq!(func.name, "add");
        assert_eq!(func.args.len(), 2);
        assert_eq!(func.args[0].name, "a");
        assert_eq!((func.args[0].ty.unwrap())().name(), "i32");
        assert_eq!(func.args[1].name, "b");
        assert_eq!((func.args[1].ty.unwrap())().name(), "i32");
        assert!(func.ret.is_some());
    }

    #[test]
    fn function_with_generics() {
        #[derive(agdb::TypeDef)]
        struct StructWithGenericFunction;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithGenericFunction {
            fn identity<T>(value: T) -> T {
                value
            }
        }

        let functions = StructWithGenericFunction::type_def().functions();

        assert_eq!(functions.len(), 1);
        let func = &functions[0];
        assert_eq!(func.name, "identity");
        assert_eq!(func.generic_params.len(), 1);
        assert_eq!(func.generic_params[0].name, "T");
        assert_eq!(func.args.len(), 1);
        assert_eq!(func.args[0].name, "value");
        assert_eq!((func.args[0].ty.unwrap())().name(), "T");
        assert!(func.ret.is_some());
    }

    #[test]
    fn impl_with_generics() {
        #[derive(agdb::TypeDef)]
        #[allow(dead_code)]
        struct StructWithGenericImpl<T> {
            value: T,
        }

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl<T> StructWithGenericImpl<T> {
            fn generic_method(value: T) -> T {
                value
            }
        }

        let functions = StructWithGenericImpl::<i32>::type_def().functions();

        assert_eq!(functions.len(), 1);
        let func = &functions[0];
        assert_eq!(func.name, "generic_method");
        assert_eq!(func.generic_params.len(), 0);
        assert_eq!(func.args.len(), 1);
        assert_eq!(func.args[0].name, "value");
        assert_eq!((func.args[0].ty.unwrap())().name(), "T");
        assert!(func.ret.is_some());
    }

    #[test]
    fn impl_with_generics_and_bounds() {
        trait Debuggable {}

        impl Debuggable for i32 {}

        #[derive(agdb::TypeDef)]
        #[allow(dead_code)]
        struct StructWithBoundedGenericImpl<T: Debuggable> {
            value: T,
        }

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl<T> StructWithBoundedGenericImpl<T>
        where
            T: Debuggable,
        {
            fn debug_value(value: T) -> T {
                value
            }
        }

        let functions = StructWithBoundedGenericImpl::<i32>::type_def().functions();

        assert_eq!(functions.len(), 1);
        let func = &functions[0];
        assert_eq!(func.name, "debug_value");
        assert_eq!(func.generic_params.len(), 0);
        assert_eq!(func.args.len(), 1);
        assert_eq!(func.args[0].name, "value");
        assert_eq!((func.args[0].ty.unwrap())().name(), "T");
        assert!(func.ret.is_some());
    }

    #[test]
    fn async_function() {
        #[derive(agdb::TypeDef)]
        struct StructWithAsync;

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithAsync {
            async fn async_function() {}
        }

        let functions = StructWithAsync::type_def().functions();

        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "async_function");
        assert!(functions[0].generic_params.is_empty());
        assert!(functions[0].args.is_empty());
        assert!(functions[0].ret.is_none());
        assert!(functions[0].async_fn);
        assert!(functions[0].expressions.is_empty());
    }

    #[test]
    fn generic_return_value() {
        #[derive(agdb::TypeDef)]
        struct StructWithGenericReturn;

        #[derive(agdb::TypeDefImpl)]
        struct GenericReturn<T> {
            value: T,
        }

        #[agdb::impl_def()]
        #[allow(dead_code)]
        impl StructWithGenericReturn {
            fn get_value() -> GenericReturn<i32> {
                GenericReturn { value: 42 }
            }
        }

        let function = &StructWithGenericReturn::type_def().functions()[0];
    }
}
