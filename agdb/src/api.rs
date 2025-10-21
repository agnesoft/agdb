use crate::SearchQuery;
use crate::query_builder::search::SearchQueryBuilder;
use agdb::ApiDef;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Clone, Debug)]
pub enum Type {
    None,
    U8,
    I64,
    U64,
    F64,
    String,
    User,
    Enum(Enum),
    Struct(Struct),
    List(List),
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for Type {}

impl std::hash::Hash for Type {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name().hash(state);
    }
}

#[derive(Clone, Debug)]
pub enum LiteralValue {
    I64(&'static str),
    F64(&'static str),
    String(&'static str),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub struct NamedType {
    pub name: &'static str,
    pub ty: fn() -> Type,
}

#[derive(Clone, Debug)]
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

#[derive(Debug, Clone)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<NamedType>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: &'static str,
    pub args: Vec<NamedType>,
    pub ret: Option<fn() -> Type>,
    pub expressions: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<NamedType>,
    pub functions: fn() -> Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct List {
    pub name: String,
    pub ty: fn() -> Type,
}

#[derive(ApiDef)]
pub struct SearchQueryBuilderHelper {
    search: SearchQuery,
}

pub trait ApiDefinition: ApiFunctions {
    fn def() -> Type;
}

impl Type {
    pub fn name(&self) -> &str {
        match self {
            Type::None => "None",
            Type::U8 => "u8",
            Type::I64 => "i64",
            Type::U64 => "u64",
            Type::F64 => "f64",
            Type::String => "String",
            Type::User => "User",
            Type::Enum(e) => &e.name,
            Type::Struct(s) => &s.name,
            Type::List(t) => &t.name,
        }
    }

    pub fn functions(&self) -> Vec<Function> {
        match self {
            Type::Struct(s) => (s.functions)(),
            _ => vec![],
        }
    }
}

impl SearchQueryBuilder for SearchQueryBuilderHelper {
    fn search_mut(&mut self) -> &mut SearchQuery {
        &mut self.search
    }
}

pub trait ApiFunctions {
    fn functions() -> Vec<Function> {
        Vec::new()
    }
}

impl ApiDefinition for u8 {
    fn def() -> Type {
        Type::U8
    }
}

impl ApiFunctions for u8 {}

impl ApiDefinition for u16 {
    fn def() -> Type {
        Type::U64
    }
}

impl ApiFunctions for u16 {}

impl ApiDefinition for i64 {
    fn def() -> Type {
        Type::I64
    }
}

impl ApiFunctions for i64 {}

impl ApiDefinition for u64 {
    fn def() -> Type {
        Type::U64
    }
}

impl ApiFunctions for u64 {}

impl ApiDefinition for f64 {
    fn def() -> Type {
        Type::F64
    }
}

impl ApiFunctions for f64 {}

impl ApiDefinition for String {
    fn def() -> Type {
        Type::String
    }
}

impl ApiFunctions for String {}

impl ApiDefinition for &str {
    fn def() -> Type {
        Type::String
    }
}

impl ApiFunctions for &str {}

impl ApiDefinition for bool {
    fn def() -> Type {
        Type::U8
    }
}

impl ApiFunctions for bool {}

impl<T: ApiDefinition> ApiDefinition for Vec<T> {
    fn def() -> Type {
        Type::List(List {
            name: format!("List_{}", T::def().name()),
            ty: || T::def(),
        })
    }
}

impl<T: ApiDefinition> ApiFunctions for Vec<T> {}

impl<T: ApiDefinition> ApiDefinition for Option<T> {
    fn def() -> Type {
        Type::Enum(Enum {
            name: format!("Option_{}", T::def().name()),
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
        })
    }
}

impl<T: ApiDefinition> ApiFunctions for Option<T> {}

impl<T: ApiDefinition, E: ApiDefinition> ApiDefinition for Result<T, E> {
    fn def() -> Type {
        Type::Enum(Enum {
            name: format!("Result_{}_{}", T::def().name(), E::def().name()),
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
        })
    }
}

impl<T: ApiDefinition, E: ApiDefinition> ApiFunctions for Result<T, E> {}

impl<T1: ApiDefinition, T2: ApiDefinition> ApiDefinition for (T1, T2) {
    fn def() -> Type {
        Type::Struct(::agdb::api::Struct {
            name: format!("Tuple_{}_{}", T1::def().name(), T2::def().name()),
            fields: vec![
                NamedType {
                    name: "",
                    ty: || T1::def(),
                },
                NamedType {
                    name: "",
                    ty: || T2::def(),
                },
            ],
            functions: || <(T1, T2) as ApiFunctions>::functions(),
        })
    }
}

impl<T1: ApiDefinition, T2: ApiDefinition> ApiFunctions for (T1, T2) {}

impl<T: ApiDefinition> ApiDefinition for &[T] {
    fn def() -> Type {
        Type::List(List {
            name: format!("List_{}", T::def().name()),
            ty: || T::def(),
        })
    }
}

impl<T: ApiDefinition> ApiFunctions for &[T] {}

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
