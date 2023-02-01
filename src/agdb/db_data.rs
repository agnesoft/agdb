use crate::graph::Graph;
use crate::DbError;

pub struct DbData {
    pub graph: Graph,
}

impl DbData {
    pub fn new() -> Result<DbData, DbError> {
        Ok(Self {
            graph: Graph::new(),
        })
    }
}
