use crate::SearchQuery;
use crate::query_builder::search::SearchQueryBuilder;
use agdb_derive::ApiDef;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Clone, Debug, PartialEq)]
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
}

#[derive(Clone, Debug, PartialEq)]
pub enum LiteralValue {
    I64(&'static str),
    F64(&'static str),
    String(&'static str),
    Bool(bool),
}

#[derive(Debug, PartialEq)]
pub struct NamedType {
    pub name: &'static str,
    pub ty: fn() -> Type,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    BitXor,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    Eq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,
    Not,
    Neg,
}

#[derive(Clone, Debug)]
pub enum Expression {
    Array {
        elements: Vec<Expression>,
    },
    Assign {
        target: Box<Expression>,
        value: Box<Expression>,
    },
    Binary {
        op: Op,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Block(Vec<Expression>),
    Call {
        recipient: Option<Box<Expression>>,
        function: &'static str,
        args: Vec<Expression>,
    },
    Closure {
        ret: Option<fn() -> Type>,
        body: Vec<Expression>,
    },
    FieldAccess {
        base: Box<Expression>,
        field: &'static str,
    },
    If {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Option<Box<Expression>>,
    },
    Index {
        base: Box<Expression>,
        index: Box<Expression>,
    },
    Let {
        name: &'static str,
        ty: Option<fn() -> Type>,
        value: Option<Box<Expression>>,
    },
    Literal(LiteralValue),
    Return(Option<Box<Expression>>),
    Struct {
        name: &'static str,
        fields: Vec<(&'static str, Box<Expression>)>,
    },
    Unary {
        op: Op,
        expr: Box<Expression>,
    },
    Variable(&'static str),
    While {
        condition: Box<Expression>,
        body: Box<Expression>,
    },
    Unknown(String),
}

#[derive(Debug, PartialEq)]
pub struct Enum {
    pub name: &'static str,
    pub variants: Vec<NamedType>,
}

#[derive(Debug)]
pub struct Function {
    pub name: &'static str,
    pub args: Vec<NamedType>,
    pub ret: Option<fn() -> Type>,
    pub expressions: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
pub struct Struct {
    pub name: &'static str,
    pub fields: Vec<NamedType>,
}

#[derive(ApiDef)]
pub struct SearchQueryBuilderHelper {
    search: SearchQuery,
}

pub trait ApiDefinition {
    fn def() -> Type;
}

impl Type {
    pub fn name(&self) -> &'static str {
        match self {
            Type::None => "None",
            Type::U8 => "u8",
            Type::I64 => "i64",
            Type::U64 => "u64",
            Type::F64 => "f64",
            Type::String => "String",
            Type::User => "User",
            Type::Enum(e) => e.name,
            Type::Struct(s) => s.name,
            Type::List(t) => {
                static LIST_NAME: std::sync::OnceLock<String> = std::sync::OnceLock::new();
                let name = LIST_NAME.get_or_init(|| format!("List_{}", t.name()));
                name.as_str()
            }
        }
    }
}

impl SearchQueryBuilder for SearchQueryBuilderHelper {
    fn search_mut(&mut self) -> &mut SearchQuery {
        &mut self.search
    }
}

pub trait ApiFunctions: ApiDefinition {
    fn functions() -> Vec<Function>;
}

impl ApiDefinition for u8 {
    fn def() -> Type {
        Type::U8
    }
}

impl ApiDefinition for u16 {
    fn def() -> Type {
        Type::U64
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

impl ApiDefinition for &str {
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
        static ENUM: std::sync::OnceLock<Enum> = std::sync::OnceLock::new();
        let e = ENUM.get_or_init(|| Enum {
            name: "Option",
            variants: vec![
                NamedType {
                    name: "Some",
                    ty: || T::def(),
                },
                NamedType {
                    name: "None",
                    ty: || Type::None,
                },
            ],
        });
        Type::Enum(e)
    }
}

impl<T: ApiDefinition, E: ApiDefinition> ApiDefinition for Result<T, E> {
    fn def() -> Type {
        static ENUM: std::sync::OnceLock<Enum> = std::sync::OnceLock::new();
        let e = ENUM.get_or_init(|| Enum {
            name: "Result",
            variants: vec![
                NamedType {
                    name: "Ok",
                    ty: || T::def(),
                },
                NamedType {
                    name: "Err",
                    ty: || E::def(),
                },
            ],
        });
        Type::Enum(e)
    }
}

impl<T1: ApiDefinition, T2: ApiDefinition> ApiDefinition for (T1, T2) {
    fn def() -> Type {
        static STRUCT: std::sync::OnceLock<agdb::api::Struct> = std::sync::OnceLock::new();
        static STRUCTNAME: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        let name = STRUCTNAME
            .get_or_init(|| format!("{}_{}", T1::def().name(), T2::def().name()))
            .as_str();

        Type::Struct(STRUCT.get_or_init(|| ::agdb::api::Struct {
            name,
            fields: vec![
                NamedType {
                    name: "0",
                    ty: || T1::def(),
                },
                NamedType {
                    name: "1",
                    ty: || T2::def(),
                },
            ],
        }))
    }
}

impl<T: ApiDefinition> ApiDefinition for &[T] {
    fn def() -> Type {
        Type::List(Box::new(T::def()))
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Op::Add => write!(f, "+"),
            Op::Sub => write!(f, "-"),
            Op::Mul => write!(f, "*"),
            Op::Div => write!(f, "/"),
            Op::Rem => write!(f, "%"),
            Op::And => write!(f, "&&"),
            Op::Or => write!(f, "||"),
            Op::BitXor => write!(f, "^"),
            Op::BitAnd => write!(f, "&"),
            Op::BitOr => write!(f, "|"),
            Op::Shl => write!(f, "<<"),
            Op::Shr => write!(f, ">>"),
            Op::Eq => write!(f, "=="),
            Op::Lt => write!(f, "<"),
            Op::Le => write!(f, "<="),
            Op::Ne => write!(f, "!="),
            Op::Ge => write!(f, ">="),
            Op::Gt => write!(f, ">"),
            Op::Not => write!(f, "!"),
            Op::Neg => write!(f, "-"),
        }
    }
}

pub struct ApiType {
    pub ty: Type,
    pub functions: Vec<Function>,
}

pub fn ty<T: ApiDefinition>() -> ApiType {
    ApiType {
        ty: T::def(),
        functions: vec![],
    }
}

pub fn ty_f<T: ApiFunctions>() -> ApiType {
    ApiType {
        ty: T::def(),
        functions: T::functions(),
    }
}
