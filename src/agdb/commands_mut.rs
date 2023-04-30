pub mod insert_alias;
pub mod insert_edge;
pub mod insert_index;
pub mod insert_node;
pub mod remove_alias;
pub mod remove_edge;
pub mod remove_edge_index;
pub mod remove_index;
pub mod remove_index_id;
pub mod remove_node;
pub mod remove_node_index;

use self::insert_alias::InsertAlias;
use self::insert_edge::InsertEdge;
use self::insert_index::InsertIndex;
use self::insert_node::InsertNode;
use self::remove_alias::RemoveAlias;
use self::remove_edge::RemoveEdge;
use self::remove_edge_index::RemoveEdgeIndex;
use self::remove_index::RemoveIndex;
use self::remove_index_id::RemoveIndexId;
use self::remove_node::RemoveNode;
use self::remove_node_index::RemoveNodeIndex;

#[derive(Debug, PartialEq)]
pub enum CommandsMut {
    None,
    InsertAlias(InsertAlias),
    InsertEdge(InsertEdge),
    InsertIndex(InsertIndex),
    InsertNode(InsertNode),
    RemoveAlias(RemoveAlias),
    RemoveEdge(RemoveEdge),
    RemoveEdgeIndex(RemoveEdgeIndex),
    RemoveNode(RemoveNode),
    RemoveNodeIndex(RemoveNodeIndex),
    RemoveIndex(RemoveIndex),
    RemoveIndexId(RemoveIndexId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            CommandsMut::InsertAlias(InsertAlias {
                id: None,
                alias: String::new(),
                result: false
            })
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            CommandsMut::InsertAlias(InsertAlias {
                id: None,
                alias: String::new(),
                result: false
            }),
            CommandsMut::InsertAlias(InsertAlias {
                id: None,
                alias: String::new(),
                result: false
            })
        );
    }
}
