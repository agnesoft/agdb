use crate::collections::indexed_map::IndexedMap;
use crate::graph::Graph;
use crate::query::query_id::QueryId;
use crate::DbError;
use crate::QueryError;

pub struct DbData {
    pub graph: Graph,
    pub aliases: IndexedMap<String, i64>,
    pub indexes: IndexedMap<i64, i64>,
    pub next_node: i64,
    pub next_edge: i64,
}

impl DbData {
    pub fn new() -> Result<DbData, DbError> {
        Ok(Self {
            graph: Graph::new(),
            aliases: IndexedMap::<String, i64>::new(),
            indexes: IndexedMap::<i64, i64>::new(),
            next_node: 1,
            next_edge: -1,
        })
    }
}

pub(crate) fn index_from_id(id: &QueryId, db_data: &DbData) -> Result<i64, QueryError> {
    Ok(match id {
        QueryId::Id(id) => {
            let _ = db_data
                .indexes
                .value(id)?
                .ok_or(QueryError::from(format!("Id '{id}' not found")))?;
            *id
        }
        QueryId::Alias(alias) => db_data
            .aliases
            .value(alias)?
            .ok_or(QueryError::from(format!("Alias '{alias}' not found")))?,
    })
}
