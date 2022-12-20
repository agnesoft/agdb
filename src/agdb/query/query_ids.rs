use super::query_id::QueryId;
use super::search_query::SearchQuery;

pub enum QueryIds {
    Ids(Vec<QueryId>),
    Search(SearchQuery),
}
