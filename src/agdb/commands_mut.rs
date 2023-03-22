pub mod insert_alias;
pub mod insert_edge;
pub mod insert_node;
pub mod remove_alias;
pub mod remove_edge;
pub mod remove_node;

use self::insert_alias::InsertAlias;
use self::insert_edge::InsertEdge;
use self::insert_node::InsertNode;
use self::remove_alias::RemoveAlias;
use self::remove_edge::RemoveEdge;
use self::remove_node::RemoveNode;

#[derive(Debug, PartialEq)]
pub enum CommandsMut {
    InsertAlias(InsertAlias),
    InsertEdge(InsertEdge),
    InsertNode(InsertNode),
    RemoveAlias(RemoveAlias),
    RemoveEdge(RemoveEdge),
    RemoveNode(RemoveNode),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", CommandsMut::InsertNode(InsertNode { alias: None }));
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            CommandsMut::InsertNode(InsertNode { alias: None }),
            CommandsMut::InsertNode(InsertNode { alias: None })
        );
    }
}
