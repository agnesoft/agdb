pub enum Type {
    None,
    U8,
    I64,
    U64,
    F64,
    String,
    User,
    Enum(&'static Enum),
    Struct(&'static Struct),
    List(Box<Type>),
    Option(Box<Type>),
}

pub struct NamedType {
    pub name: &'static str,
    pub ty: fn() -> Type,
}

pub enum Expression {
    Create {
        ty: NamedType,
    },
    CreateArg {
        ty: NamedType,
        arg: &'static str,
    },
    CreateReturn {
        ty: fn() -> Type,
    },
    CreateReturnArg {
        ty: fn() -> Type,
        arg: &'static str,
    },
    CreateReturnArgT {
        ty: fn() -> Type,
        arg: &'static str,
    },
    Assign {
        object: &'static str,
        fields: Vec<&'static str>,
        value: &'static str,
    },
    Return(&'static str),
}

pub struct Enum {
    pub name: &'static str,
    pub variants: Vec<NamedType>,
}

pub struct Function {
    pub name: &'static str,
    pub args: Vec<NamedType>,
    pub ret: fn() -> Type,
    pub expressions: Vec<Expression>,
}

pub struct Struct {
    pub name: &'static str,
    pub fields: Vec<NamedType>,
}

pub trait ApiDefinition {
    fn def() -> Type;
}

pub trait ApiFunctions: ApiDefinition {
    fn functions() -> Vec<Function>;
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

impl Expression {
    pub fn create(name: &'static str, ty: fn() -> Type) -> Self {
        Expression::Create {
            ty: NamedType { name, ty },
        }
    }

    pub fn create_arg(name: &'static str, ty: fn() -> Type, arg: &'static str) -> Self {
        Expression::CreateArg {
            ty: NamedType { name, ty },
            arg,
        }
    }

    pub fn create_return(ty: fn() -> Type) -> Self {
        Expression::CreateReturn { ty }
    }

    pub fn create_return_arg(ty: fn() -> Type, arg: &'static str) -> Self {
        Expression::CreateReturnArg { ty, arg }
    }

    pub fn create_return_arg_t(ty: fn() -> Type, arg: &'static str) -> Self {
        Expression::CreateReturnArgT { ty, arg }
    }

    pub fn assign(object: &'static str, field: &'static str, value: &'static str) -> Self {
        Expression::Assign {
            object,
            fields: vec![field],
            value,
        }
    }

    pub fn assign_fields(
        object: &'static str,
        fields: Vec<&'static str>,
        value: &'static str,
    ) -> Self {
        Expression::Assign {
            object,
            fields,
            value,
        }
    }

    pub fn ret(value: &'static str) -> Self {
        Expression::Return(value)
    }
}
