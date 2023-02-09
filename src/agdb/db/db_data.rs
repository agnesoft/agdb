use crate::collections::indexed_map::IndexedMap;
use crate::graph::Graph;
use crate::DbError;

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
