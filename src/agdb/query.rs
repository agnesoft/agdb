pub mod comparison_operator;
pub mod condition;
pub mod direction;
pub mod distance_condition;
pub mod edge_count_condition;
pub mod insert_aliases_query;
pub mod insert_edges_query;
pub mod insert_nodes_query;
pub mod insert_query;
pub mod key_value_condition;
pub mod query_error;
pub mod query_id;
pub mod query_ids;
pub mod query_result;
pub mod query_values;
pub mod search_query;
pub mod select_query;

use self::insert_aliases_query::InsertAliasQuery;
use self::insert_edges_query::InsertEdgesQuery;
use self::insert_nodes_query::InsertNodesQuery;
use self::insert_query::InsertQuery;
use self::query_ids::QueryIds;
use self::search_query::SearchQuery;
use self::select_query::SelectQuery;

pub enum QueryData {
    Insert(InsertQuery),
    InsertAliases(InsertAliasQuery),
    InsertEdges(InsertEdgesQuery),
    InsertNodes(InsertNodesQuery),
    Remove(SelectQuery),
    Search(SearchQuery),
    Select(SelectQuery),
    SelectAliases(QueryIds),
    SelectCount(SearchQuery),
    SelectKeyCount(QueryIds),
    SelectKeys(QueryIds),
}

pub trait Query {
    fn data(self) -> QueryData;
}
