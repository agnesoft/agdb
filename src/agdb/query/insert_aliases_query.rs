use super::query_ids::QueryIds;

pub struct InsertAliasQuery {
    pub ids: QueryIds,
    pub aliases: Vec<String>,
}
