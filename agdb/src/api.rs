pub use agdb_derive::ApiDef;

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct NamedType {
    pub name: &'static str,
    pub ty: Type,
}

#[derive(Debug, PartialEq)]
pub struct Enum {
    pub name: &'static str,
    pub variants: Vec<NamedType>,
}

#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_def() {
        assert_eq!(u8::def(), Type::U8);
        assert_eq!(i64::def(), Type::I64);
        assert_eq!(u64::def(), Type::U64);
        assert_eq!(f64::def(), Type::F64);
        assert_eq!(String::def(), Type::String);
        assert_eq!(bool::def(), Type::U8);
        assert_eq!(Vec::<u8>::def(), Type::List(Box::new(Type::U8)));
        assert_eq!(Option::<u8>::def(), Type::Option(Box::new(Type::U8)));
    }
}
