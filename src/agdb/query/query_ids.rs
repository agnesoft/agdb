use super::search_query::SearchQuery;

#[allow(dead_code)]
pub enum QueryIds {
    Ids(Vec<u64>),
    Search(SearchQuery),
}
