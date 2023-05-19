mod insert;
mod insert_alias;
mod insert_edge;
mod insert_node;
mod insert_values;
mod remove;
mod remove_alias;
mod remove_ids;
mod remove_values;
mod search;
mod select;
mod select_alias;
mod select_ids;
mod select_key_count;
mod select_keys;
mod select_values;
mod where_;
mod where_key;
mod where_logic_operator;

use self::insert::Insert;
use self::remove::Remove;
use self::search::Search;
use self::select::Select;

pub struct QueryBuilder {}

impl QueryBuilder {
    pub fn insert() -> Insert {
        Insert {}
    }

    pub fn remove() -> Remove {
        Remove {}
    }

    pub fn search() -> Search {
        Search {}
    }

    pub fn select() -> Select {
        Select {}
    }
}
