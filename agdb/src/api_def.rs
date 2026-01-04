pub mod enum_def;
pub mod expression_def;
pub mod function_def;
pub mod struct_def;
pub mod tuple_def;

pub use enum_def::Enum;
pub use expression_def::Expression;
pub use expression_def::Literal;
pub use expression_def::Op;
pub use function_def::Function;
pub use struct_def::Struct;
pub use tuple_def::Tuple;

pub trait ImplDefinition {
    fn functions() -> &'static [Function] {
        &[]
    }
}

pub trait TypeDefinition: ImplDefinition {
    fn type_def() -> Type;
    fn generic_type_names() -> Vec<&'static str> {
        match Self::type_def() {
            Type::Enum(e) => e.generics.iter().map(|g| g.name).collect(),
            Type::Struct(s) => s.generics.iter().map(|g| g.name).collect(),
            Type::Tuple(t) => t.generics.iter().map(|g| g.name).collect(),
        }
    }
}

pub enum Type {
    Enum(Enum),
    Struct(Struct),
    Tuple(Tuple),
}

impl Type {
    #[allow(dead_code)]
    pub fn functions(&self) -> &'static [Function] {
        match self {
            Type::Enum(e) => e.functions,
            Type::Struct(s) => s.functions,
            Type::Tuple(t) => t.functions,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Type::Enum(e) => e.name,
            Type::Struct(s) => s.name,
            Type::Tuple(t) => t.name,
        }
    }
    pub fn generics(&self) -> &[Generic] {
        match self {
            Type::Enum(e) => e.generics,
            Type::Struct(s) => s.generics,
            Type::Tuple(t) => t.generics,
        }
    }
}

pub struct NamedType {
    pub name: &'static str,
    pub ty: Option<fn() -> Type>,
}

pub struct Generic {
    pub name: &'static str,
    pub bounds: &'static [&'static str],
}

// --- Rust types implementations --- //

#[macro_export]
macro_rules! impl_type {
    ($ty:ty) => {
        impl $crate::api_def::ImplDefinition for $ty {}

        impl $crate::api_def::TypeDefinition for $ty {
            fn type_def() -> $crate::api_def::Type {
                $crate::api_def::Type::Struct($crate::api_def::struct_def::Struct {
                    name: stringify!($ty),
                    generics: &[],
                    fields: &[],
                    functions: &[],
                })
            }
        }
    };
}

impl_type!(bool);
impl_type!(i8);
impl_type!(i16);
impl_type!(i32);
impl_type!(i64);
impl_type!(u8);
impl_type!(u16);
impl_type!(u32);
impl_type!(u64);
impl_type!(f64);
impl_type!(String);
impl_type!(&str);
impl_type!(());

impl<T: TypeDefinition> TypeDefinition for Option<T> {
    fn type_def() -> Type {
        Type::Struct(Struct {
            name: "Option",
            generics: &[Generic {
                name: "T",
                bounds: &[],
            }],
            fields: &[],
            functions: &[],
        })
    }

    fn generic_type_names() -> Vec<&'static str> {
        vec![T::type_def().name()]
    }
}

impl<T: ImplDefinition> ImplDefinition for Option<T> {
    fn functions() -> &'static [Function] {
        &[]
    }
}

impl<T: TypeDefinition> TypeDefinition for Vec<T> {
    fn type_def() -> Type {
        Type::Struct(Struct {
            name: "Vec",
            generics: &[Generic {
                name: "T",
                bounds: &[],
            }],
            fields: &[],
            functions: &[],
        })
    }

    fn generic_type_names() -> Vec<&'static str> {
        vec![T::type_def().name()]
    }
}

impl<T: ImplDefinition> ImplDefinition for Vec<T> {
    fn functions() -> &'static [Function] {
        &[]
    }
}

impl<T: TypeDefinition, E: TypeDefinition> TypeDefinition for Result<T, E> {
    fn type_def() -> Type {
        Type::Struct(Struct {
            name: "Result",
            generics: &[
                Generic {
                    name: "T",
                    bounds: &[],
                },
                Generic {
                    name: "E",
                    bounds: &[],
                },
            ],
            fields: &[],
            functions: &[],
        })
    }

    fn generic_type_names() -> Vec<&'static str> {
        vec![T::type_def().name(), E::type_def().name()]
    }
}

impl<T: ImplDefinition, E: ImplDefinition> ImplDefinition for Result<T, E> {
    fn functions() -> &'static [Function] {
        &[]
    }
}

impl<T1: TypeDefinition, T2: TypeDefinition> TypeDefinition for (T1, T2) {
    fn type_def() -> Type {
        Type::Tuple(Tuple {
            name: "Tuple2",
            generics: &[
                Generic {
                    name: "T1",
                    bounds: &[],
                },
                Generic {
                    name: "T2",
                    bounds: &[],
                },
            ],
            fields: &[],
            functions: &[],
        })
    }

    fn generic_type_names() -> Vec<&'static str> {
        vec![T1::type_def().name(), T2::type_def().name()]
    }
}

impl<T1: ImplDefinition, T2: ImplDefinition> ImplDefinition for (T1, T2) {
    fn functions() -> &'static [Function] {
        &[]
    }
}

impl<T: TypeDefinition> TypeDefinition for &[T] {
    fn type_def() -> Type {
        Type::Struct(Struct {
            name: "Slice",
            generics: &[Generic {
                name: "T",
                bounds: &[],
            }],
            fields: &[],
            functions: &[],
        })
    }

    fn generic_type_names() -> Vec<&'static str> {
        vec![T::type_def().name()]
    }
}

impl<T: ImplDefinition> ImplDefinition for &[T] {
    fn functions() -> &'static [Function] {
        &[]
    }
}
