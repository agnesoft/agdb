//! Persistent embedded memory mapped graph database with native object queries.
//!
//! [Readme](https://github.com/agnesoft/agdb) |
//! [Quickstart](https://github.com/agnesoft/agdb#quickstart) |
//! [Queries](https://github.com/agnesoft/agdb/blob/main/docs/queries.md) |
//! [Efficient agdb](https://github.com/agnesoft/agdb/blob/main/docs/efficient_agdb.md)
//!
//! # Example
//!
//! ```
//! use agdb::{Db, QueryBuilder};
//!
//! let mut db = Db::new("db.agdb").unwrap();
//! db.exec_mut(&QueryBuilder::insert().nodes().values(vec![vec![("key", 123).into()]]).query()).unwrap();
//!
//! let result = db.exec(&QueryBuilder::select().ids(1).query()).unwrap();
//! println!("{:?}", result);
//! // QueryResult { result: 1, elements: [ DbElement { id: DbId(1), values: [ DbKeyValue { key: String("key"), value: Int(123) } ] } ] }
//! ```

mod collections;
mod command;
mod db;
mod graph;
mod graph_search;
mod query;
mod query_builder;
mod storage;
mod transaction;
mod transaction_mut;
mod utilities;

#[cfg(test)]
pub mod test_utilities;

#[cfg(any(feature = "serde", feature = "opeanapi"))]
pub use query::QueryType;

#[cfg(feature = "derive")]
pub use agdb_derive::UserValue;

pub use db::db_element::DbElement;
pub use db::db_error::DbError;
pub use db::db_f64::DbF64;
pub use db::db_id::DbId;
pub use db::db_key::DbKeyOrder;
pub use db::db_key_value::DbKeyValue;
pub use db::db_user_value::DbUserValue;
pub use db::db_value::DbValue;
pub use db::Db;
pub use db::DbFile;
pub use db::DbFileTransaction;
pub use db::DbFileTransactionMut;
pub use db::DbImpl;
pub use db::DbMemory;
pub use db::DbMemoryTransaction;
pub use db::DbMemoryTransactionMut;
pub use db::DbTransaction;
pub use db::DbTransactionMut;
pub use query::insert_aliases_query::InsertAliasesQuery;
pub use query::insert_edges_query::InsertEdgesQuery;
pub use query::insert_index_query::InsertIndexQuery;
pub use query::insert_nodes_query::InsertNodesQuery;
pub use query::insert_values_query::InsertValuesQuery;
pub use query::query_condition::Comparison;
pub use query::query_condition::CountComparison;
pub use query::query_condition::QueryCondition;
pub use query::query_condition::QueryConditionData;
pub use query::query_condition::QueryConditionLogic;
pub use query::query_condition::QueryConditionModifier;
pub use query::query_error::QueryError;
pub use query::query_id::QueryId;
pub use query::query_ids::QueryIds;
pub use query::query_result::QueryResult;
pub use query::query_values::QueryValues;
pub use query::remove_aliases_query::RemoveAliasesQuery;
pub use query::remove_index_query::RemoveIndexQuery;
pub use query::remove_query::RemoveQuery;
pub use query::remove_values_query::RemoveValuesQuery;
pub use query::search_query::SearchQuery;
pub use query::search_query::SearchQueryAlgorithm;
pub use query::select_aliases_query::SelectAliasesQuery;
pub use query::select_all_aliases_query::SelectAllAliasesQuery;
pub use query::select_edge_count_query::SelectEdgeCountQuery;
pub use query::select_indexes_query::SelectIndexesQuery;
pub use query::select_key_count_query::SelectKeyCountQuery;
pub use query::select_keys_query::SelectKeysQuery;
pub use query::select_query::SelectQuery;
pub use query::select_values_query::SelectValuesQuery;
pub use query::Query;
pub use query::QueryMut;
pub use query_builder::QueryBuilder;
pub use storage::file_storage::FileStorage;
pub use storage::file_storage_memory_mapped::FileStorageMemoryMapped;
pub use storage::memory_storage::MemoryStorage;
pub use storage::StorageData;
pub use storage::StorageSlice;
pub use transaction::Transaction;
pub use transaction_mut::TransactionMut;
pub use utilities::serialize::Serialize as AgdbSerialize;
pub use utilities::stable_hash::StableHash;
