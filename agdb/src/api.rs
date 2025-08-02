use crate::Comparison;
use crate::CountComparison;
use crate::DbF64;
use crate::DbId;
use crate::DbKeyOrder;
use crate::DbKeyValue;
use crate::DbValue;
use crate::DbValues;
use crate::InsertAliasesQuery;
use crate::InsertEdgesQuery;
use crate::InsertIndexQuery;
use crate::InsertNodesQuery;
use crate::InsertValuesQuery;
use crate::KeyValueComparison;
use crate::MultiValues;
use crate::QueryAliases;
use crate::QueryBuilder;
use crate::QueryCondition;
use crate::QueryConditionData;
use crate::QueryConditionLogic;
use crate::QueryConditionModifier;
use crate::QueryId;
use crate::QueryIds;
use crate::QueryValues;
use crate::RemoveAliasesQuery;
use crate::RemoveIndexQuery;
use crate::RemoveQuery;
use crate::RemoveValuesQuery;
use crate::SearchQuery;
use crate::SearchQueryAlgorithm;
use crate::SelectAliasesQuery;
use crate::SelectAllAliasesQuery;
use crate::SelectEdgeCountQuery;
use crate::SelectIndexesQuery;
use crate::SelectKeyCountQuery;
use crate::SelectKeysQuery;
use crate::SelectNodeCountQuery;
use crate::SelectValuesQuery;
use crate::SingleValues;
use crate::query_builder::insert::Insert;
use crate::query_builder::insert_aliases::InsertAliases;
use crate::query_builder::insert_aliases::InsertAliasesIds;
use crate::query_builder::insert_edge::InsertEdges;
use crate::query_builder::insert_edge::InsertEdgesEach;
use crate::query_builder::insert_edge::InsertEdgesFrom;
use crate::query_builder::insert_edge::InsertEdgesFromTo;
use crate::query_builder::insert_edge::InsertEdgesIds;
use crate::query_builder::insert_edge::InsertEdgesValues;
use crate::query_builder::insert_index::InsertIndex;
use crate::query_builder::insert_nodes::InsertNodes;
use crate::query_builder::insert_nodes::InsertNodesAliases;
use crate::query_builder::insert_nodes::InsertNodesCount;
use crate::query_builder::insert_nodes::InsertNodesIds;
use crate::query_builder::insert_nodes::InsertNodesValues;
use crate::query_builder::insert_values::InsertValues;
use crate::query_builder::insert_values::InsertValuesIds;
use crate::query_builder::remove::Remove;
use crate::query_builder::remove_aliases::RemoveAliases;
use crate::query_builder::remove_ids::RemoveIds;
use crate::query_builder::remove_index::RemoveIndex;
use crate::query_builder::remove_values::RemoveValues;
use crate::query_builder::remove_values::RemoveValuesIds;
use crate::query_builder::search::Search;
use crate::query_builder::search::SearchAlgorithm;
use crate::query_builder::search::SearchFrom;
use crate::query_builder::search::SearchIndex;
use crate::query_builder::search::SearchIndexValue;
use crate::query_builder::search::SearchOrderBy;
use crate::query_builder::search::SearchQueryBuilder;
use crate::query_builder::search::SearchTo;
use crate::query_builder::search::SelectLimit;
use crate::query_builder::search::SelectOffset;
use crate::query_builder::select::Select;
use crate::query_builder::select_aliases::SelectAliases;
use crate::query_builder::select_aliases::SelectAliasesIds;
use crate::query_builder::select_edge_count::SelectEdgeCount;
use crate::query_builder::select_edge_count::SelectEdgeCountIds;
use crate::query_builder::select_ids::SelectIds;
use crate::query_builder::select_indexes::SelectIndexes;
use crate::query_builder::select_key_count::SelectKeyCount;
use crate::query_builder::select_key_count::SelectKeyCountIds;
use crate::query_builder::select_keys::SelectKeys;
use crate::query_builder::select_keys::SelectKeysIds;
use crate::query_builder::select_node_count::SelectNodeCount;
use crate::query_builder::select_values::SelectValues;
use crate::query_builder::select_values::SelectValuesIds;
use crate::query_builder::where_::Where;
use crate::query_builder::where_::WhereKey;
use crate::query_builder::where_::WhereLogicOperator;
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
    Option(Box<Type>),
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

#[allow(dead_code)]
#[derive(ApiDef)]
struct SearchQueryBuilderHelper {
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
            Type::List(_) => "List",
            Type::Option(_) => "Option",
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
        Type::Option(Box::new(T::def()))
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

#[allow(dead_code)]
pub struct ApiType {
    pub ty: Type,
    pub functions: Vec<Function>,
}

#[allow(dead_code, clippy::upper_case_acronyms)]
pub struct API {
    pub types: Vec<ApiType>,
}

#[allow(dead_code)]
fn ty<T: ApiDefinition>() -> ApiType {
    ApiType {
        ty: T::def(),
        functions: vec![],
    }
}

#[allow(dead_code)]
fn ty_f<T: ApiFunctions>() -> ApiType {
    ApiType {
        ty: T::def(),
        functions: T::functions(),
    }
}

#[allow(dead_code)]
impl API {
    pub fn def() -> Self {
        Self {
            types: vec![
                //literals
                ty::<u8>(),
                ty::<i64>(),
                ty::<u64>(),
                ty::<f64>(),
                ty::<String>(),
                ty::<bool>(),
                ty::<Vec<u8>>(),
                ty::<DbF64>(),
                //structs
                ty::<DbId>(),
                ty::<QueryId>(),
                ty::<QueryIds>(),
                ty::<QueryValues>(),
                ty::<DbValue>(),
                ty::<DbValues>(),
                ty::<DbKeyValue>(),
                ty::<QueryAliases>(),
                ty::<SingleValues>(),
                ty::<MultiValues>(),
                //queries
                ty::<InsertAliasesQuery>(),
                ty::<InsertEdgesQuery>(),
                ty::<InsertIndexQuery>(),
                ty::<InsertNodesQuery>(),
                ty::<InsertValuesQuery>(),
                ty::<RemoveAliasesQuery>(),
                ty::<RemoveIndexQuery>(),
                ty::<RemoveQuery>(),
                ty::<RemoveValuesQuery>(),
                ty::<SearchQuery>(),
                ty::<SearchQueryAlgorithm>(),
                ty::<SelectAliasesQuery>(),
                ty::<SelectAllAliasesQuery>(),
                ty::<SelectEdgeCountQuery>(),
                ty::<SelectIndexesQuery>(),
                ty::<SelectKeyCountQuery>(),
                ty::<SelectKeysQuery>(),
                ty::<SelectNodeCountQuery>(),
                ty::<SelectValuesQuery>(),
                ty::<DbKeyOrder>(),
                ty::<QueryCondition>(),
                ty::<QueryConditionLogic>(),
                ty::<QueryConditionModifier>(),
                ty::<QueryConditionData>(),
                ty::<CountComparison>(),
                ty::<Comparison>(),
                ty::<KeyValueComparison>(),
                //builders
                ty_f::<QueryBuilder>(),
                ty_f::<Insert>(),
                ty_f::<InsertAliases>(),
                ty_f::<InsertAliasesIds>(),
                ty_f::<InsertEdges>(),
                ty_f::<InsertEdgesEach>(),
                ty_f::<InsertEdgesFrom>(),
                ty_f::<InsertEdgesFromTo>(),
                ty_f::<InsertEdgesIds>(),
                ty_f::<InsertEdgesValues>(),
                ty_f::<InsertIndex>(),
                ty_f::<InsertNodes>(),
                ty_f::<InsertNodesAliases>(),
                ty_f::<InsertNodesCount>(),
                ty_f::<InsertNodesIds>(),
                ty_f::<InsertNodesValues>(),
                ty_f::<InsertValues>(),
                ty_f::<InsertValuesIds>(),
                ty_f::<Remove>(),
                ty_f::<RemoveAliases>(),
                ty_f::<RemoveIds>(),
                ty_f::<RemoveIndex>(),
                ty_f::<RemoveValues>(),
                ty_f::<RemoveValuesIds>(),
                ty_f::<Select>(),
                ty_f::<SelectAliases>(),
                ty_f::<SelectAliasesIds>(),
                ty_f::<SelectEdgeCount>(),
                ty_f::<SelectEdgeCountIds>(),
                ty_f::<SelectIds>(),
                ty_f::<SelectIndexes>(),
                ty_f::<SelectKeys>(),
                ty_f::<SelectKeysIds>(),
                ty_f::<SelectKeyCount>(),
                ty_f::<SelectKeyCountIds>(),
                ty_f::<SelectNodeCount>(),
                ty_f::<SelectValues>(),
                ty_f::<SelectValuesIds>(),
                //search & where
                ty_f::<Search<SearchQueryBuilderHelper>>(),
                ty_f::<SearchAlgorithm<SearchQueryBuilderHelper>>(),
                ty_f::<SearchFrom<SearchQueryBuilderHelper>>(),
                ty_f::<SearchTo<SearchQueryBuilderHelper>>(),
                ty_f::<SearchIndex<SearchQueryBuilderHelper>>(),
                ty_f::<SearchIndexValue<SearchQueryBuilderHelper>>(),
                ty_f::<SearchOrderBy<SearchQueryBuilderHelper>>(),
                ty_f::<SelectLimit<SearchQueryBuilderHelper>>(),
                ty_f::<SelectOffset<SearchQueryBuilderHelper>>(),
                ty_f::<Where<SearchQueryBuilderHelper>>(),
                ty_f::<WhereKey<SearchQueryBuilderHelper>>(),
                ty_f::<WhereLogicOperator<SearchQueryBuilderHelper>>(),
                ty::<SearchQueryBuilderHelper>(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api() {
        let api = API::def();
        for ty in api.types {
            for f in ty.functions {
                for e in f.expressions {
                    if let Expression::Unknown(e) = e {
                        panic!("Unknown expression in {:?}::{}: {}", ty.ty, f.name, e);
                    }
                }
            }
        }
    }
}
