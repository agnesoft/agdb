use crate::graph::graph_index::GraphIndex;
use crate::graph::Graph;
use crate::DbError;
use std::collections::HashMap;

pub struct DbData {
    pub graph: Graph,
    pub aliases: HashMap<String, GraphIndex>,
}

impl DbData {
    pub fn new() -> Result<DbData, DbError> {
        Ok(Self {
            graph: Graph::new(),
            aliases: HashMap::<String, GraphIndex>::new(),
        })
    }
}
