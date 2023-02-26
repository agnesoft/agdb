pub mod insert_alias;
pub mod insert_edge;
pub mod insert_node;
pub mod remove_alias;
pub mod remove_edge;
pub mod remove_node;
pub mod select_id;

use self::insert_alias::InsertAlias;
use self::insert_edge::InsertEdge;
use self::insert_node::InsertNode;
use self::remove_alias::RemoveAlias;
use self::remove_edge::RemoveEdge;
use self::remove_node::RemoveNode;
use self::select_id::SelectId;

#[derive(Debug, PartialEq)]
pub enum Commands {
    InsertAlias(InsertAlias),
    InsertEdge(InsertEdge),
    InsertNode(InsertNode),
    RemoveAlias(RemoveAlias),
    RemoveEdge(RemoveEdge),
    RemoveNode(RemoveNode),
    SelectId(SelectId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", Commands::InsertNode(InsertNode { alias: None }));
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            Commands::InsertNode(InsertNode { alias: None }),
            Commands::InsertNode(InsertNode { alias: None })
        );
    }
}
