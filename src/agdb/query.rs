mod insert_edges_query;
mod insert_nodes_query;
mod insert_values_query;
mod query_error;
mod query_ids;
mod query_result;
mod query_values;
mod search_query;
mod select_keys_query;

use self::insert_edges_query::InsertEdgesQuery;
use self::insert_nodes_query::InsertNodeQuery;
use self::insert_values_query::InsertValuesQuery;
use self::query_ids::QueryIds;
use self::query_values::QueryValues;
use self::search_query::SearchQuery;
use self::select_keys_query::SelectKeysQuery;
pub use query_error::QueryError;
pub use query_result::QueryResult;

pub enum Query {
    InsertEdges(InsertEdgesQuery),
    InsertNodes(InsertNodeQuery),
    InsertValues(InsertValuesQuery),
    SelectAliases(QueryIds),
    SelectElements(QueryIds),
    SelectKeys(SelectKeysQuery),
    SelectKeyCount(QueryIds),
    SelectCount(SearchQuery),
    Search(SearchQuery),
}

impl Default for Query {
    fn default() -> Self {
        Query::InsertNodes(InsertNodeQuery {
            count: 0,
            values: QueryValues::None,
        })
    }
}
