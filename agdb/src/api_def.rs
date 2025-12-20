pub mod enum_def;
pub mod expression_def;
pub mod function_def;
pub mod struct_def;
pub mod trait_def;
pub mod tuple_def;

pub use enum_def::Enum;
pub use expression_def::Expression;
pub use expression_def::Literal;
pub use expression_def::Op;
pub use function_def::Function;
pub use struct_def::Struct;
pub use trait_def::Trait;
pub use tuple_def::Tuple;

pub trait ImplDefinition {
    fn functions() -> &'static [Function] {
        &[]
    }
}

pub trait TypeDefinition: ImplDefinition {
    fn type_def() -> Type;
}

pub enum Type {
    Enum(Enum),
    Struct(Struct),
    Tuple(Tuple),
}

impl Type {
    pub fn name(&self) -> &'static str {
        match self {
            Type::Enum(e) => e.name,
            Type::Struct(s) => s.name,
            Type::Tuple(t) => t.name,
        }
    }

    pub fn functions(&self) -> &'static [Function] {
        match self {
            Type::Enum(e) => e.functions,
            Type::Struct(s) => s.functions,
            Type::Tuple(t) => t.functions,
        }
    }
}

pub struct NamedType {
    pub name: &'static str,
    pub ty: Option<fn() -> Type>,
}

pub struct Generic {
    pub name: &'static str,
    pub bounds: &'static [fn() -> Trait],
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
impl_type!(i32);
impl_type!(i64);
impl_type!(f64);
impl_type!(String);
impl_type!(Vec<i32>);
impl_type!(Result<i32, String>);
