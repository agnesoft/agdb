pub mod insert_alias;
pub mod insert_edge;
pub mod insert_node;
pub mod remove_alias;
pub mod remove_edge;
pub mod remove_index;
pub mod remove_node;

use self::insert_alias::InsertAlias;
use self::insert_edge::InsertEdge;
use self::insert_node::InsertNode;
use self::remove_alias::RemoveAlias;
use self::remove_edge::RemoveEdge;
use self::remove_index::RemoveIndex;
use self::remove_node::RemoveNode;

#[derive(Debug, PartialEq)]
pub enum CommandsMut {
    InsertAlias(InsertAlias),
    InsertEdge(InsertEdge),
    InsertNode(InsertNode),
    RemoveAlias(RemoveAlias),
    RemoveEdge(RemoveEdge),
    RemoveNode(RemoveNode),
    RemoveIndex(RemoveIndex),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::query_id::QueryId;
    use crate::DbId;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            CommandsMut::InsertAlias(InsertAlias::new(QueryId::Id(DbId(0)), String::new()))
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            CommandsMut::InsertAlias(InsertAlias::new(QueryId::Id(DbId(0)), String::new())),
            CommandsMut::InsertAlias(InsertAlias::new(QueryId::Id(DbId(0)), String::new()))
        );
    }
}
