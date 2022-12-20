mod insert_aliases_query;
mod insert_edges_query;
mod insert_nodes_query;
mod insert_query;
mod query_error;
mod query_id;
mod query_ids;
mod query_result;
mod query_values;
mod search_query;
mod select_query;

use self::insert_aliases_query::InsertAliasQuery;
use self::insert_edges_query::InsertEdgesQuery;
use self::insert_nodes_query::InsertNodeQuery;
use self::insert_query::InsertQuery;
use self::query_ids::QueryIds;
use self::query_values::QueryValues;
use self::search_query::SearchQuery;
use self::select_query::SelectQuery;
pub use query_error::QueryError;
pub use query_result::QueryResult;

pub enum Query {
    Insert(InsertQuery),
    InsertAliases(InsertAliasQuery),
    InsertEdges(InsertEdgesQuery),
    InsertNodes(InsertNodeQuery),
    Remove(SelectQuery),
    Search(SearchQuery),
    Select(SelectQuery),
    SelectAliases(QueryIds),
    SelectCount(SearchQuery),
    SelectKeyCount(QueryIds),
    SelectKeys(QueryIds),
}

impl Default for Query {
    fn default() -> Self {
        Query::InsertNodes(InsertNodeQuery {
            count: 0,
            values: QueryValues::None,
            aliases: vec![],
        })
    }
}
