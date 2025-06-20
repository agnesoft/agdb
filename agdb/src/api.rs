use agdb_derive::ApiDef;

use crate::DbId;
use crate::DbKeyValue;
use crate::DbValue;
use crate::DbValues;
use crate::InsertAliasesQuery;
use crate::InsertEdgesQuery;
use crate::InsertIndexQuery;
use crate::InsertNodesQuery;
use crate::InsertValuesQuery;
use crate::MultiValues;
use crate::QueryAliases;
use crate::QueryBuilder;
use crate::QueryId;
use crate::QueryIds;
use crate::QueryValues;
use crate::RemoveAliasesQuery;
use crate::RemoveIndexQuery;
use crate::RemoveQuery;
use crate::RemoveValuesQuery;
use crate::SearchQuery;
use crate::SelectAliasesQuery;
use crate::SelectAllAliasesQuery;
use crate::SelectIndexesQuery;
use crate::SelectKeyCountQuery;
use crate::SelectKeysQuery;
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct NamedType {
    pub name: &'static str,
    pub ty: fn() -> Type,
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug)]
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
        value: Box<Expression>,
    },
    Literal(Type),
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
    pub ret: fn() -> Type,
    pub expressions: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
pub struct Struct {
    pub name: &'static str,
    pub fields: Vec<NamedType>,
}

#[allow(dead_code)]
#[derive(ApiDef)]
struct SearchQueryBuilderDummy {
    search: SearchQuery,
}

pub trait ApiDefinition {
    fn def() -> Type;
}

impl SearchQueryBuilder for SearchQueryBuilderDummy {
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

#[allow(dead_code)]
struct ApiType {
    ty: Type,
    functions: Vec<Function>,
}

#[allow(dead_code, clippy::upper_case_acronyms)]
struct API {
    types: Vec<ApiType>,
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
    pub fn new() -> Self {
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
                ty::<SelectAliasesQuery>(),
                ty::<SelectAllAliasesQuery>(),
                ty::<SelectEdgeCount>(),
                ty::<SelectIndexesQuery>(),
                ty::<SelectKeyCountQuery>(),
                ty::<SelectKeysQuery>(),
                ty::<SelectNodeCount>(),
                ty::<SelectValuesQuery>(),
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
                ty_f::<Search<SearchQueryBuilderDummy>>(),
                ty_f::<SearchAlgorithm<SearchQueryBuilderDummy>>(),
                ty_f::<SearchFrom<SearchQueryBuilderDummy>>(),
                ty_f::<SearchTo<SearchQueryBuilderDummy>>(),
                ty_f::<SearchIndex<SearchQueryBuilderDummy>>(),
                ty_f::<SearchIndexValue<SearchQueryBuilderDummy>>(),
                ty_f::<SearchOrderBy<SearchQueryBuilderDummy>>(),
                ty_f::<SelectLimit<SearchQueryBuilderDummy>>(),
                ty_f::<SelectOffset<SearchQueryBuilderDummy>>(),
                ty_f::<Where<SearchQueryBuilderDummy>>(),
                ty_f::<WhereKey<SearchQueryBuilderDummy>>(),
                ty_f::<WhereLogicOperator<SearchQueryBuilderDummy>>(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api() {
        let _api = API::new();
        for ty in _api.types {
            for f in ty.functions {
                for e in f.expressions {
                    if let Expression::Unknown(a) = e {
                        println!("{a}");
                    }
                }
            }
        }
    }
}
