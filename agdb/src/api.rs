use crate::DbId;
use crate::DbKeyValue;
use crate::DbValue;
use crate::DbValues;
use crate::InsertAliasesQuery;
use crate::InsertEdgesQuery;
use crate::InsertNodesQuery;
use crate::InsertValuesQuery;
use crate::MultiValues;
use crate::QueryAliases;
use crate::QueryBuilder;
use crate::QueryId;
use crate::QueryIds;
use crate::QueryValues;
use crate::RemoveValuesQuery;
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
use crate::query_builder::select::Select;
use crate::query_builder::select_aliases::SelectAliases;
use crate::query_builder::select_edge_count::SelectEdgeCount;
use crate::query_builder::select_ids::SelectIds;
use crate::query_builder::select_indexes::SelectIndexes;
use crate::query_builder::select_key_count::SelectKeyCount;
use crate::query_builder::select_keys::SelectKeys;
use crate::query_builder::select_node_count::SelectNodeCount;
use crate::query_builder::select_values::SelectValues;

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

#[derive(Debug, PartialEq)]
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

struct ApiType {
    ty: Type,
    functions: Vec<Function>,
}

struct API {
    types: Vec<ApiType>,
}

fn ty<T: ApiDefinition>() -> ApiType {
    ApiType {
        ty: T::def(),
        functions: vec![],
    }
}

fn ty_f<T: ApiFunctions>() -> ApiType {
    ApiType {
        ty: T::def(),
        functions: T::functions(),
    }
}

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
                ty::<InsertNodesQuery>(),
                ty::<InsertValuesQuery>(),
                ty::<RemoveValuesQuery>(),
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
                ty_f::<SelectEdgeCount>(),
                ty_f::<SelectIds>(),
                ty_f::<SelectIndexes>(),
                ty_f::<SelectKeys>(),
                ty_f::<SelectKeyCount>(),
                ty_f::<SelectNodeCount>(),
                ty_f::<SelectValues>(),
            ],
        }
    }
}
