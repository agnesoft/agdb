mod insert;
mod insert_alias;
mod insert_aliases;
mod insert_aliases_of;
mod insert_edge;
mod insert_edge_from;
mod insert_edge_from_to;
mod insert_edge_values;
mod insert_edges;
mod insert_edges_each;
mod insert_edges_from;
mod insert_edges_from_to;
mod insert_edges_values;
mod insert_node;
mod insert_node_alias;
mod insert_node_values;
mod insert_nodes;
mod insert_nodes_aliases;
mod insert_nodes_count;
mod insert_nodes_values;
mod insert_values;
mod insert_values_into;
mod insert_values_multi;
mod remove;
mod remove_alias;
mod remove_ids;
mod remove_values;
mod remove_values_from;
mod select;
mod select_alias;
mod select_aliases;
mod select_aliases_of;
mod select_from;

use self::insert::Insert;
use self::remove::Remove;
use self::select::Select;

pub struct QueryBuilder {}

impl QueryBuilder {
    pub fn insert() -> Insert {
        Insert {}
    }

    pub fn remove() -> Remove {
        Remove {}
    }

    pub fn select() -> Select {
        Select {}
    }
}
