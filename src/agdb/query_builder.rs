mod insert;
mod insert_edge;
mod insert_edge_from;
mod insert_edge_from_to;
mod insert_edge_values;
mod insert_node;
mod insert_node_alias;
mod insert_node_values;
mod insert_nodes;
mod insert_nodes_aliases;
mod insert_nodes_count;
mod insert_nodes_values;
mod select;
mod select_from;

use self::insert::InsertBuilder;
use self::select::Select;

pub struct QueryBuilder {}

impl QueryBuilder {
    pub fn insert() -> InsertBuilder {
        InsertBuilder {}
    }

    pub fn select() -> Select {
        Select {}
    }
}
