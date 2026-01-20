pub mod enum_def;
pub mod expression_def;
pub mod function_def;
pub mod struct_def;
pub mod tuple_struct_def;

pub use enum_def::Enum;
pub use expression_def::Expression;
pub use expression_def::Literal;
pub use expression_def::Op;
pub use function_def::Function;
pub use struct_def::Struct;
pub use tuple_struct_def::TupleStruct;

pub trait ImplDefinition {
    fn functions() -> &'static [Function] {
        &[]
    }
}

pub trait TypeDefinition: ImplDefinition {
    fn type_def() -> Type;
}

#[derive(Debug)]
pub enum LiteralType {
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    String,
    Str,
    Unit,
}

#[derive(Debug)]
pub enum Type {
    Literal(LiteralType),
    Enum(Enum),
    Struct(Struct),
    TupleStruct(TupleStruct),
    Tuple(&'static [fn() -> Type]),
    Slice(fn() -> Type),
    Vec(fn() -> Type),
    Option(fn() -> Type),
    Result(fn() -> Type, fn() -> Type),
    GenericArg(GenericArg),
}

impl Type {
    pub fn functions(&self) -> &'static [Function] {
        match self {
            Type::Enum(e) => e.functions,
            Type::Struct(s) => s.functions,
            Type::TupleStruct(t) => t.functions,
            _ => &[],
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Type::Enum(e) => e.name,
            Type::Struct(s) => s.name,
            Type::TupleStruct(t) => t.name,
            _ => "",
        }
    }
}

#[derive(Debug)]
pub struct NamedType {
    pub name: &'static str,
    pub ty: Option<fn() -> Type>,
}

#[derive(Debug)]
pub struct GenericParam {
    pub name: &'static str,
    pub bounds: &'static [&'static Trait],
}

#[derive(Debug)]
pub struct GenericArg {
    pub name: &'static str,
    pub args: &'static [fn() -> Type],
}

#[derive(Debug)]
pub struct Trait {
    pub name: &'static str,
    pub bounds: &'static [&'static Trait],
    pub generic_params: &'static [GenericParam],
    pub functions: &'static [Function],
}

// --- Rust types implementations --- //

#[macro_export]
macro_rules! impl_type_literal {
    ($ty:ty, $literal:path) => {
        impl $crate::api_def::ImplDefinition for $ty {}

        impl $crate::api_def::TypeDefinition for $ty {
            fn type_def() -> $crate::api_def::Type {
                $crate::api_def::Type::Literal($literal)
            }
        }
    };
}

impl_type_literal!(bool, LiteralType::Bool);
impl_type_literal!(i8, LiteralType::I8);
impl_type_literal!(i16, LiteralType::I16);
impl_type_literal!(i32, LiteralType::I32);
impl_type_literal!(i64, LiteralType::I64);
impl_type_literal!(u8, LiteralType::U8);
impl_type_literal!(u16, LiteralType::U16);
impl_type_literal!(u32, LiteralType::U32);
impl_type_literal!(u64, LiteralType::U64);
impl_type_literal!(f64, LiteralType::F64);
impl_type_literal!(String, LiteralType::String);
impl_type_literal!(&str, LiteralType::Str);
impl_type_literal!((), LiteralType::Unit);

impl<T: TypeDefinition> TypeDefinition for Option<T> {
    fn type_def() -> Type {
        Type::Option(T::type_def)
    }
}

impl<T: ImplDefinition> ImplDefinition for Option<T> {
    fn functions() -> &'static [Function] {
        &[]
    }
}

impl<T: TypeDefinition> TypeDefinition for Vec<T> {
    fn type_def() -> Type {
        Type::Vec(T::type_def)
    }
}

impl<T: ImplDefinition> ImplDefinition for Vec<T> {
    fn functions() -> &'static [Function] {
        &[]
    }
}

impl<T: TypeDefinition, E: TypeDefinition> TypeDefinition for Result<T, E> {
    fn type_def() -> Type {
        Type::Result(T::type_def, E::type_def)
    }
}

impl<T: ImplDefinition, E: ImplDefinition> ImplDefinition for Result<T, E> {
    fn functions() -> &'static [Function] {
        &[]
    }
}

impl<T1: TypeDefinition, T2: TypeDefinition> TypeDefinition for (T1, T2) {
    fn type_def() -> Type {
        Type::Tuple(&[T1::type_def, T2::type_def])
    }
}

impl<T1: ImplDefinition, T2: ImplDefinition> ImplDefinition for (T1, T2) {
    fn functions() -> &'static [Function] {
        &[]
    }
}

impl<T: TypeDefinition> TypeDefinition for &[T] {
    fn type_def() -> Type {
        Type::Slice(T::type_def)
    }
}

impl<T: ImplDefinition> ImplDefinition for &[T] {
    fn functions() -> &'static [Function] {
        &[]
    }
}
