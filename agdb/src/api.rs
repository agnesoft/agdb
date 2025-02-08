pub use agdb_derive::ApiDef;

pub enum Type {
    None,
    U8,
    I64,
    U64,
    F64,
    String,
    Enum(&'static Enum),
    Struct(&'static Struct),
    List(Box<Type>),
    Option(Box<Type>),
}

pub struct NamedType {
    pub name: &'static str,
    pub ty: Type,
}

pub struct Enum {
    pub name: &'static str,
    pub variants: Vec<NamedType>,
}

pub struct Struct {
    pub name: &'static str,
    pub fields: Vec<NamedType>,
}

pub trait ApiDefinition {
    fn def() -> Type;
}

impl ApiDefinition for u8 {
    fn def() -> Type {
        Type::U8
    }
}

impl ApiDefinition for i64 {
    fn def() -> Type {
        Type::I64
    }
}

impl ApiDefinition for u64 {
    fn def() -> Type {
        Type::U64
    }
}

impl ApiDefinition for f64 {
    fn def() -> Type {
        Type::F64
    }
}

impl ApiDefinition for String {
    fn def() -> Type {
        Type::String
    }
}

impl ApiDefinition for bool {
    fn def() -> Type {
        Type::U8
    }
}

impl<T: ApiDefinition> ApiDefinition for Vec<T> {
    fn def() -> Type {
        Type::List(Box::new(T::def()))
    }
}

impl<T: ApiDefinition> ApiDefinition for Option<T> {
    fn def() -> Type {
        Type::Option(Box::new(T::def()))
    }
}
