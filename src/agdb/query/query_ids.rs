use super::query_id::QueryId;
use super::search_query::SearchQuery;

pub enum QueryIds {
    Id(QueryId),
    Ids(Vec<QueryId>),
    Search(SearchQuery),
}
