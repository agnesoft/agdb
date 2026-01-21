pub mod enum_def;
pub mod expression_def;
pub mod function_def;
pub mod struct_def;
pub mod tuple_struct_def;
use crate::SearchQueryBuilder;
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
            Type::Literal(literal_type) => match literal_type {
                LiteralType::Bool => "bool",
                LiteralType::I8 => "i8",
                LiteralType::I16 => "i16",
                LiteralType::I32 => "i32",
                LiteralType::I64 => "i64",
                LiteralType::U8 => "u8",
                LiteralType::U16 => "u16",
                LiteralType::U32 => "u32",
                LiteralType::U64 => "u64",
                LiteralType::F32 => "f32",
                LiteralType::F64 => "f64",
                LiteralType::String => "String",
                LiteralType::Str => "&str",
                LiteralType::Unit => "()",
            },
            Type::Tuple(_) => "Tuple",
            Type::Slice(_) => "Slice",
            Type::Vec(_) => "Vec",
            Type::Option(_) => "Option",
            Type::Result(_, _) => "Result",
            Type::GenericArg(generic_arg) => generic_arg.name,
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

// --- agdb types --- //

#[derive(agdb::TypeDefImpl)]
struct SearchQueryBuilderT;

impl SearchQueryBuilder for SearchQueryBuilderT {
    fn search_mut(&mut self) -> &mut agdb::SearchQuery {
        unimplemented!()
    }
}

pub fn type_defs() -> Vec<Type> {
    vec![
        agdb::Comparison::type_def(),
        agdb::CountComparison::type_def(),
        agdb::DbElement::type_def(),
        agdb::DbF64::type_def(),
        agdb::DbId::type_def(),
        agdb::DbKeyOrder::type_def(),
        agdb::DbKeyValue::type_def(),
        agdb::DbValue::type_def(),
        agdb::InsertAliasesQuery::type_def(),
        agdb::InsertEdgesQuery::type_def(),
        agdb::InsertIndexQuery::type_def(),
        agdb::InsertNodesQuery::type_def(),
        agdb::InsertValuesQuery::type_def(),
        agdb::KeyValueComparison::type_def(),
        agdb::QueryAliases::type_def(),
        agdb::QueryBuilder::type_def(),
        agdb::QueryCondition::type_def(),
        agdb::QueryConditionData::type_def(),
        agdb::QueryConditionLogic::type_def(),
        agdb::QueryConditionModifier::type_def(),
        agdb::QueryId::type_def(),
        agdb::QueryIds::type_def(),
        agdb::QueryResult::type_def(),
        agdb::QueryType::type_def(),
        agdb::QueryValues::type_def(),
        agdb::RemoveAliasesQuery::type_def(),
        agdb::RemoveIndexQuery::type_def(),
        agdb::RemoveQuery::type_def(),
        agdb::RemoveValuesQuery::type_def(),
        agdb::SearchQuery::type_def(),
        agdb::SearchQueryAlgorithm::type_def(),
        agdb::SelectAliasesQuery::type_def(),
        agdb::SelectAllAliasesQuery::type_def(),
        agdb::SelectEdgeCountQuery::type_def(),
        agdb::SelectIndexesQuery::type_def(),
        agdb::SelectKeyCountQuery::type_def(),
        agdb::SelectKeysQuery::type_def(),
        agdb::SelectNodeCountQuery::type_def(),
        agdb::SelectValuesQuery::type_def(),
        crate::db::db_value::DbValues::type_def(),
        crate::query_builder::insert_aliases::InsertAliases::type_def(),
        crate::query_builder::insert_aliases::InsertAliasesIds::type_def(),
        crate::query_builder::insert_edge::InsertEdges::type_def(),
        crate::query_builder::insert_edge::InsertEdgesEach::type_def(),
        crate::query_builder::insert_edge::InsertEdgesFrom::type_def(),
        crate::query_builder::insert_edge::InsertEdgesFromTo::type_def(),
        crate::query_builder::insert_edge::InsertEdgesIds::type_def(),
        crate::query_builder::insert_edge::InsertEdgesValues::type_def(),
        crate::query_builder::insert_index::InsertIndex::type_def(),
        crate::query_builder::insert_nodes::InsertNodes::type_def(),
        crate::query_builder::insert_nodes::InsertNodesAliases::type_def(),
        crate::query_builder::insert_nodes::InsertNodesCount::type_def(),
        crate::query_builder::insert_nodes::InsertNodesIds::type_def(),
        crate::query_builder::insert_nodes::InsertNodesValues::type_def(),
        crate::query_builder::insert_values::InsertValues::type_def(),
        crate::query_builder::insert_values::InsertValuesIds::type_def(),
        crate::query_builder::insert::Insert::type_def(),
        crate::query_builder::remove_aliases::RemoveAliases::type_def(),
        crate::query_builder::remove_ids::RemoveIds::type_def(),
        crate::query_builder::remove_index::RemoveIndex::type_def(),
        crate::query_builder::remove_values::RemoveValues::type_def(),
        crate::query_builder::remove_values::RemoveValuesIds::type_def(),
        crate::query_builder::remove::Remove::type_def(),
        crate::query_builder::select_aliases::SelectAliases::type_def(),
        crate::query_builder::select_aliases::SelectAliasesIds::type_def(),
        crate::query_builder::select_edge_count::SelectEdgeCount::type_def(),
        crate::query_builder::select_edge_count::SelectEdgeCountIds::type_def(),
        crate::query_builder::select_ids::SelectIds::type_def(),
        crate::query_builder::select_indexes::SelectIndexes::type_def(),
        crate::query_builder::select_key_count::SelectKeyCount::type_def(),
        crate::query_builder::select_key_count::SelectKeyCountIds::type_def(),
        crate::query_builder::select_keys::SelectKeys::type_def(),
        crate::query_builder::select_keys::SelectKeysIds::type_def(),
        crate::query_builder::select_node_count::SelectNodeCount::type_def(),
        crate::query_builder::select_values::SelectValues::type_def(),
        crate::query_builder::select_values::SelectValuesIds::type_def(),
        crate::query_builder::select::Select::type_def(),
        crate::query::query_values::MultiValues::type_def(),
        crate::query::query_values::SingleValues::type_def(),
        crate::Search::<SearchQueryBuilderT>::type_def(),
    ]
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
