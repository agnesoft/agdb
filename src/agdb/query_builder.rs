mod insert;
mod insert_alias;
mod insert_edge;
mod insert_node;
mod insert_values;
mod insert_values_ids;
mod insert_values_multi;
mod remove;
mod remove_alias;
mod remove_ids;
mod remove_values;
mod remove_values_ids;
mod search;
mod search_from;
mod search_to;
mod select;
mod select_alias;
mod select_aliases;
mod select_aliases_ids;
mod select_ids;
mod select_key_count;
mod select_key_count_ids;
mod select_keys;
mod select_keys_ids;
mod select_limit;
mod select_offset;
mod select_values;
mod select_values_ids;
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
