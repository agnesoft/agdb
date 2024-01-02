use crate::graph::GraphIndex;
use crate::DbId;
use crate::DbKeyValue;
use crate::DbValue;

pub enum Command {
    InsertAlias {
        alias: String,
        id: DbId,
    },
    InsertEdge {
        from: GraphIndex,
        to: GraphIndex,
    },
    InsertIndex {
        key: DbValue,
    },
    InsertToIndex {
        key: DbValue,
        value: DbValue,
        id: DbId,
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
    RemoveIndex {
        key: DbValue,
    },
    RemoveKeyValue {
        id: DbId,
        key_value: DbKeyValue,
    },
    RemoveNode {
        index: GraphIndex,
    },
    ReplaceKeyValue {
        id: DbId,
        key_value: DbKeyValue,
    },
}
