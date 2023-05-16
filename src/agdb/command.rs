use crate::collections::dictionary::dictionary_index::DictionaryIndex;
use crate::db::db_key_value_index::DbKeyValueIndex;
use crate::graph::graph_index::GraphIndex;
use crate::DbId;
use crate::DbKeyValue;

pub(crate) enum Command {
    InsertAlias {
        id: DbId,
        alias: String,
    },
    InsertEdge {
        from: GraphIndex,
        to: GraphIndex,
    },
    InsertId {
        id: DbId,
        graph_index: GraphIndex,
    },
    InsertNode,
    InsertKeyValue {
        id: DbId,
        key_value: DbKeyValue,
    },
    NextId {
        id: i64,
    },
    RemoveAlias {
        alias: String,
    },
    RemoveId {
        id: DbId,
    },
    RemoveEdge {
        index: GraphIndex,
    },
    RemoveKeyValue {
        id: DbId,
        key_value: DbKeyValueIndex,
    },
    RemoveNode {
        index: GraphIndex,
    },
    RemoveValue {
        index: DictionaryIndex,
    },
}
