//! Persistent embedded memory mapped graph database with native object queries.
//!
//! [Readme](https://github.com/agnesoft/agdb) |
//! [Quickstart](https://github.com/agnesoft/agdb#quickstart) |
//! [Queries](https://github.com/agnesoft/agdb/blob/main/docs/queries.md) |
//! [Efficient agdb](https://github.com/agnesoft/agdb/blob/main/docs/efficient_agdb.md)
//!
//! # Features
//!
//! `agdb` ships with a small core and a set of optional features that unlock
//! additional ergonomics and integrations:
//!
//! - `derive` (default): enables `DbType`, `DbElement`, `DbValue`, and the
//!   related derive macros for model definitions.
//! - `openapi`: adds `utoipa` schema support for query builders and generated API
//!   documentation.
//! - `serde`: serializes/deserializes queries and `QueryResult` via `serde`.
//! - `regex`: enables regex-based string comparisons such as
//!   `where_().key("name").regex("^user_")`.
//!
//! # Example
//!
//! ```
//! # let _test_file = agdb::test_utilities::test_file::TestFile::from("db4.agdb");
//! use agdb::{Db, QueryBuilder};
//!
//! let db = Db::new("db4.agdb").unwrap();
//! db.exec_mut(QueryBuilder::insert().nodes().values([[("key", 123).into()]]).query()).unwrap();
//!
//! let result = db.exec(QueryBuilder::select().ids(1).query()).unwrap();
//! println!("{:?}", result);
//! // QueryResult { result: 1, elements: [ DbElement { id: DbId(1), values: [ DbKeyValue { key: String("key"), value: Int(123) } ] } ] }
//! ```

extern crate self as agdb;

mod collections;
mod db;
mod graph_search;
mod query;
mod query_builder;
mod storage;
mod transaction;
mod transaction_mut;
mod utilities;

//#[cfg(any(test, doctest))] //TODO: Enable once doctest is stabilised
pub mod test_utilities;

#[cfg(any(feature = "serde", feature = "openapi"))]
pub use query::QueryType;

#[cfg(feature = "derive")]
pub use agdb_derive::{DbElement, DbSerialize, DbType, DbTypeMarker, DbValue};

pub use db::DB_VERSION;
pub use db::Db;
pub use db::DbAny;
pub use db::DbAnyTransaction;
pub use db::DbAnyTransactionMut;
pub use db::DbFile;
pub use db::DbFileTransaction;
pub use db::DbFileTransactionMut;
pub use db::DbImpl;
pub use db::DbMemory;
pub use db::DbMemoryTransaction;
pub use db::DbMemoryTransactionMut;
pub use db::DbTransaction;
pub use db::DbTransactionMut;
pub use db::db_element::DbElement;
pub use db::db_error::DbError;
pub use db::db_error::DbErrorCategory;
pub use db::db_error::DbErrorType;
pub use db::db_f64::DbF64;
pub use db::db_id::DbId;
pub use db::db_key_order::DbKeyOrder;
pub use db::db_key_order::DbKeyOrders;
pub use db::db_key_value::DbKeyValue;
pub use db::db_migration::DbMigration;
#[doc(hidden)]
pub use db::db_type::DbFields;
pub use db::db_type::DbType;
pub use db::db_type::DbTypeMarker;
pub use db::db_value::DbValue;
pub use query::Query;
pub use query::QueryMut;
pub use query::amend::Amend;
pub use query::amend::BitwiseOp;
pub use query::insert_aliases_query::InsertAliasesQuery;
pub use query::insert_edges_query::InsertEdgesQuery;
pub use query::insert_index_query::InsertIndexQuery;
pub use query::insert_nodes_query::InsertNodesQuery;
pub use query::insert_values_query::InsertValuesQuery;
pub use query::query_condition::Comparison;
pub use query::query_condition::CountComparison;
pub use query::query_condition::KeyValueComparison;
pub use query::query_condition::QueryCondition;
pub use query::query_condition::QueryConditionData;
pub use query::query_condition::QueryConditionLogic;
pub use query::query_condition::QueryConditionModifier;
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
pub use query::select_all_edge_count_query::SelectAllEdgeCountQuery;
pub use query::select_edge_count_query::SelectEdgeCountQuery;
pub use query::select_indexes_query::SelectIndexesQuery;
pub use query::select_key_count_query::SelectKeyCountQuery;
pub use query::select_keys_query::SelectKeysQuery;
pub use query::select_node_count::SelectNodeCountQuery;
pub use query::select_values_query::SelectValuesQuery;
pub use query_builder::QueryBuilder;
pub use storage::DEFAULT_OVERLAY_LIMIT;
pub use storage::StorageConfig;
pub use storage::StorageData;
pub use storage::StorageSlice;
pub use storage::StorageSnapshot;
pub use storage::SyncMode;
pub use storage::any_storage::AnyStorage;
pub use storage::any_storage::AnyStorageImpl;
pub use storage::file_snapshot::FileSnapshot;
pub use storage::file_storage::FileStorage;
pub use storage::file_storage::FileStorageImpl;
pub use storage::file_storage_memory_mapped::FileStorageMemoryMapped;
pub use storage::file_storage_memory_mapped::FileStorageMemoryMappedImpl;
pub use storage::memory_storage::MemoryStorage;
pub use storage::memory_storage::MemoryStorageImpl;
pub use storage::segments::Segments;
pub use transaction::Transaction;
pub use transaction_mut::TransactionMut;
pub use utilities::serialize::Serialize as AgdbSerialize;
pub use utilities::stable_hash::StableHash;
