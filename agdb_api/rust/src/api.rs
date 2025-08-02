use agdb::Comparison;
use agdb::CountComparison;
use agdb::DbF64;
use agdb::DbId;
use agdb::DbKeyOrder;
use agdb::DbKeyValue;
use agdb::DbValue;
use agdb::DbValues;
use agdb::InsertAliasesQuery;
use agdb::InsertEdgesQuery;
use agdb::InsertIndexQuery;
use agdb::InsertNodesQuery;
use agdb::InsertValuesQuery;
use agdb::KeyValueComparison;
use agdb::MultiValues;
use agdb::QueryAliases;
use agdb::QueryBuilder;
use agdb::QueryCondition;
use agdb::QueryConditionData;
use agdb::QueryConditionLogic;
use agdb::QueryConditionModifier;
use agdb::QueryId;
use agdb::QueryIds;
use agdb::QueryValues;
use agdb::RemoveAliasesQuery;
use agdb::RemoveIndexQuery;
use agdb::RemoveQuery;
use agdb::RemoveValuesQuery;
use agdb::SearchQuery;
use agdb::SearchQueryAlgorithm;
use agdb::SelectAliasesQuery;
use agdb::SelectAllAliasesQuery;
use agdb::SelectEdgeCountQuery;
use agdb::SelectIndexesQuery;
use agdb::SelectKeyCountQuery;
use agdb::SelectKeysQuery;
use agdb::SelectNodeCountQuery;
use agdb::SelectValuesQuery;
use agdb::SingleValues;
use agdb::api::ApiType;
use agdb::api::SearchQueryBuilderHelper;
use agdb::api::ty;
use agdb::api::ty_f;
use agdb::query_builder::insert::Insert;
use agdb::query_builder::insert_aliases::InsertAliases;
use agdb::query_builder::insert_aliases::InsertAliasesIds;
use agdb::query_builder::insert_edge::InsertEdges;
use agdb::query_builder::insert_edge::InsertEdgesEach;
use agdb::query_builder::insert_edge::InsertEdgesFrom;
use agdb::query_builder::insert_edge::InsertEdgesFromTo;
use agdb::query_builder::insert_edge::InsertEdgesIds;
use agdb::query_builder::insert_edge::InsertEdgesValues;
use agdb::query_builder::insert_index::InsertIndex;
use agdb::query_builder::insert_nodes::InsertNodes;
use agdb::query_builder::insert_nodes::InsertNodesAliases;
use agdb::query_builder::insert_nodes::InsertNodesCount;
use agdb::query_builder::insert_nodes::InsertNodesIds;
use agdb::query_builder::insert_nodes::InsertNodesValues;
use agdb::query_builder::insert_values::InsertValues;
use agdb::query_builder::insert_values::InsertValuesIds;
use agdb::query_builder::remove::Remove;
use agdb::query_builder::remove_aliases::RemoveAliases;
use agdb::query_builder::remove_ids::RemoveIds;
use agdb::query_builder::remove_index::RemoveIndex;
use agdb::query_builder::remove_values::RemoveValues;
use agdb::query_builder::remove_values::RemoveValuesIds;
use agdb::query_builder::search::Search;
use agdb::query_builder::search::SearchAlgorithm;
use agdb::query_builder::search::SearchFrom;
use agdb::query_builder::search::SearchIndex;
use agdb::query_builder::search::SearchIndexValue;
use agdb::query_builder::search::SearchOrderBy;
use agdb::query_builder::search::SearchTo;
use agdb::query_builder::search::SelectLimit;
use agdb::query_builder::search::SelectOffset;
use agdb::query_builder::select::Select;
use agdb::query_builder::select_aliases::SelectAliases;
use agdb::query_builder::select_aliases::SelectAliasesIds;
use agdb::query_builder::select_edge_count::SelectEdgeCount;
use agdb::query_builder::select_edge_count::SelectEdgeCountIds;
use agdb::query_builder::select_ids::SelectIds;
use agdb::query_builder::select_indexes::SelectIndexes;
use agdb::query_builder::select_key_count::SelectKeyCount;
use agdb::query_builder::select_key_count::SelectKeyCountIds;
use agdb::query_builder::select_keys::SelectKeys;
use agdb::query_builder::select_keys::SelectKeysIds;
use agdb::query_builder::select_node_count::SelectNodeCount;
use agdb::query_builder::select_values::SelectValues;
use agdb::query_builder::select_values::SelectValuesIds;
use agdb::query_builder::where_::Where;
use agdb::query_builder::where_::WhereKey;
use agdb::query_builder::where_::WhereLogicOperator;

use crate::AgdbApi;
use crate::ReqwestClient;

#[allow(dead_code, clippy::upper_case_acronyms)]
pub struct API {
    pub types: Vec<ApiType>,
}

impl API {
    #[allow(dead_code)]
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
                //api
                ty_f::<AgdbApi<ReqwestClient>>(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agdb::api::Expression;

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
