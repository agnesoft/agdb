pub mod insert_aliases_query;
pub mod insert_edges_query;
pub mod insert_nodes_query;
pub mod insert_values_query;
pub mod query_aliases;
pub mod query_condition;
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
pub mod select_all_aliases_query;
pub mod select_key_count_query;
pub mod select_keys_query;
pub mod select_query;
pub mod select_values_query;

use crate::Db;
use crate::QueryError;
use crate::QueryResult;

/// Trait for immutable `agdb` database queries. This
/// trait is unlikely to be implementable for user types.
pub trait Query {
    fn process(&self, db: &Db) -> Result<QueryResult, QueryError>;
}

/// Trait for mutable `agdb` database queries. This
/// trait is unlikely to be implementable for user types.
pub trait QueryMut {
    fn process(&self, db: &mut Db) -> Result<QueryResult, QueryError>;
}