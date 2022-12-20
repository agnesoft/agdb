use super::query_ids::QueryIds;

pub struct InsertAliasQuery {
    pub id: QueryIds,
    pub aliases: Vec<String>,
}
