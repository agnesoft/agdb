use crate::graph::graph_index::GraphIndex;
use crate::DbId;

pub(crate) struct Context {
    pub(crate) id: DbId,
    pub(crate) graph_index: GraphIndex,
}
