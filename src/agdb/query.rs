pub mod comparison_operator;
pub mod condition;
pub mod direction;
pub mod distance_condition;
pub mod edge_count_condition;
pub mod insert_aliases_query;
pub mod insert_edges_query;
pub mod insert_nodes_query;
pub mod insert_values_query;
pub mod key_value_condition;
pub mod query_error;
pub mod query_id;
pub mod query_ids;
pub mod query_result;
pub mod query_values;
pub mod remove_aliases_query;
pub mod remove_query;
pub mod remove_values_query;
pub mod search_query;
pub mod select_aliases_query;
pub mod select_count_query;
pub mod select_key_count_query;
pub mod select_keys_query;
pub mod select_query;

pub trait Query {}
