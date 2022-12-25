mod insert;
mod insert_node;
mod insert_node_alias;
mod insert_node_values;

use self::insert::InsertBuilder;

pub struct QueryBuilder {}

impl QueryBuilder {
    pub fn insert() -> InsertBuilder {
        InsertBuilder {}
    }
}
