use crate::collections::dictionary::dictionary_index::DictionaryIndex;
use crate::db::db_key_value_index::DbKeyValueIndex;
use crate::graph::GraphIndex;
use crate::DbId;
use crate::DbKeyValue;

pub(crate) enum Command {
    InsertAlias {
        alias: String,
        id: DbId,
    },
    InsertEdge {
        from: GraphIndex,
        to: GraphIndex,
    },
    InsertNode,
    InsertKeyValue {
        id: DbId,
        key_value: DbKeyValue,
    },
    RemoveAlias {
        alias: String,
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
