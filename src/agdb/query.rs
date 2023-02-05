pub mod comparison;
pub mod condition;
pub mod direction;
pub mod edge_count_condition;
pub mod insert_aliases_query;
pub mod insert_edges_query;
pub mod insert_nodes_query;
pub mod insert_values_query;
pub mod key_value_condition;
pub mod query_error;
pub mod query_id;
pub mod query_ids;
pub mod query_result;
pub mod query_values;
pub mod remove_aliases_query;
pub mod remove_query;
pub mod remove_values_query;
pub mod search_query;
pub mod select_aliases_query;
pub mod select_key_count_query;
pub mod select_keys_query;
pub mod select_query;
pub mod select_values_query;

use self::insert_aliases_query::InsertAliasesQuery;
use self::insert_edges_query::InsertEdgesQuery;
use self::insert_nodes_query::InsertNodesQuery;
use self::insert_values_query::InsertValuesQuery;
use self::remove_aliases_query::RemoveAliasesQuery;
use self::remove_query::RemoveQuery;
use self::remove_values_query::RemoveValuesQuery;
use self::search_query::SearchQuery;
use self::select_aliases_query::SelectAliasesQuery;
use self::select_key_count_query::SelectKeyCountQuery;
use self::select_keys_query::SelectKeysQuery;
use self::select_query::SelectQuery;
use self::select_values_query::SelectValuesQuery;
use crate::commands::Commands;

pub enum Query {
    InsertAliases(InsertAliasesQuery),
    InsertEdges(InsertEdgesQuery),
    InsertNodes(InsertNodesQuery),
    InsertValues(InsertValuesQuery),
    RemoveAliases(RemoveAliasesQuery),
    Remove(RemoveQuery),
    RemoveValues(RemoveValuesQuery),
    Search(SearchQuery),
    SelectAliases(SelectAliasesQuery),
    SelectKeys(SelectKeysQuery),
    SelectKeyCount(SelectKeyCountQuery),
    Select(SelectQuery),
    SelectValues(SelectValuesQuery),
}

impl Query {
    pub fn commands(&self) -> Vec<Commands> {
        match self {
            Query::InsertAliases(_) => todo!(),
            Query::InsertEdges(_) => todo!(),
            Query::InsertNodes(query) => query.commands(),
            Query::InsertValues(_) => todo!(),
            Query::RemoveAliases(_) => todo!(),
            Query::Remove(_) => todo!(),
            Query::RemoveValues(_) => todo!(),
            Query::Search(_) => todo!(),
            Query::SelectAliases(_) => todo!(),
            Query::SelectKeys(_) => todo!(),
            Query::SelectKeyCount(_) => todo!(),
            Query::Select(_) => todo!(),
            Query::SelectValues(_) => todo!(),
        }
    }
}
