use crate::{
    collections::dictionary::dictionary_index::DictionaryIndex, graph::graph_index::GraphIndex,
    DbId, DbValue,
};

pub(crate) enum Command {
    InsertAlias { id: DbId, alias: String },
    InsertEdge { from: GraphIndex, to: GraphIndex },
    InsertId { id: DbId, graph_index: GraphIndex },
    InsertNode,
    InsertValue { value: DbValue },
    NextId { id: i64 },
    RemoveAlias { alias: String },
    RemoveId { id: DbId },
    RemoveEdge { index: GraphIndex },
    RemoveNode { index: GraphIndex },
    RemoveValue { index: DictionaryIndex },
}
