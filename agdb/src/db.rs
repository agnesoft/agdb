pub mod db_element;
pub mod db_error;
pub mod db_f64;
pub mod db_id;
pub mod db_index;
pub mod db_key;
pub mod db_key_value;
pub mod db_user_value;
pub mod db_value;

mod db_search_handlers;
mod db_value_index;

use self::db_error::DbError;
use self::db_search_handlers::DefaultHandler;
use self::db_search_handlers::LimitHandler;
use self::db_search_handlers::LimitOffsetHandler;
use self::db_search_handlers::OffsetHandler;
use self::db_search_handlers::PathHandler;
use crate::collections::indexed_map::DbIndexedMap;
use crate::collections::multi_map::MultiMapStorage;
use crate::command::Command;
use crate::db::db_index::DbIndexes;
use crate::graph::DbGraph;
use crate::graph::GraphIndex;
use crate::graph_search::GraphSearch;
use crate::graph_search::SearchControl;
use crate::query::query_condition::QueryCondition;
use crate::query::query_condition::QueryConditionData;
use crate::query::query_condition::QueryConditionLogic;
use crate::query::query_condition::QueryConditionModifier;
use crate::query::query_id::QueryId;
use crate::query::Query;
use crate::query::QueryMut;
use crate::storage::file_storage::FileStorage;
use crate::storage::file_storage_memory_mapped::FileStorageMemoryMapped;
use crate::storage::memory_storage::MemoryStorage;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use crate::DbId;
use crate::DbKeyValue;
use crate::DbValue;
use crate::QueryError;
use crate::QueryResult;
use crate::SearchQueryAlgorithm;
use crate::StorageData;
use crate::Transaction;
use crate::TransactionMut;

#[derive(Default)]
struct DbStorageIndex {
    graph: StorageIndex,
    aliases: (StorageIndex, StorageIndex),
    indexes: StorageIndex,
    values: StorageIndex,
}

impl Serialize for DbStorageIndex {
    fn serialize(&self) -> Vec<u8> {
        [
            self.graph.serialize(),
            self.aliases.0.serialize(),
            self.aliases.1.serialize(),
            self.indexes.serialize(),
            self.values.serialize(),
        ]
        .concat()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let size = i64::serialized_size_static() as usize;

        let graph = StorageIndex::deserialize(bytes)?;
        let aliases_1 = StorageIndex::deserialize(&bytes[size..])?;
        let aliases_2 = StorageIndex::deserialize(&bytes[size * 2..])?;
        let indexes = StorageIndex::deserialize(&bytes[size * 3..])?;
        let values = StorageIndex::deserialize(&bytes[size * 4..])?;

        Ok(Self {
            graph,
            aliases: (aliases_1, aliases_2),
            indexes,
            values,
        })
    }

    fn serialized_size(&self) -> u64 {
        i64::serialized_size_static() * 5
    }
}

/// An instance of the `agdb` database. To create a database:
///
/// ```
/// use agdb::Db;
///
/// let mut db = Db::new("db.agdb").unwrap();
/// ```
///
/// This will try to create or load the database file path `db.agdb`.
/// If the file does not exist a new database will be initialized creating
/// the given file. If the file does exist the database will try to load
/// it and memory map the data.
///
/// These are the available variants of the database to choose from:
///
/// - [`Db`]: \[default] File based and memory mapped database.
/// - [`DbFile`]: File based only (no memory mapping).
/// - [`DbMemory`]: In-memory database only.
///
/// For each of these there are convenient using declarations, e.g. `DbTransaction`,
/// `DbFileTransaction`, `DbMemoryTransactionMut` etc. in case you need to name
/// the related types of the main database type.
///
/// You can execute queries or transactions on the database object with
///
/// - [exec()](#method.exec) //immutable queries
/// - [exec_mut()](#method.exec_mut) //mutable queries
/// - [transaction()](#method.transaction) //immutable transactions
/// - [transaction_mut()](#method.transaction_mut) // mutable transaction
///
/// # Examples
///
/// ```
/// use agdb::{Db, QueryBuilder, QueryError};
///
/// let mut db = Db::new("db.agdb").unwrap();
///
/// // Insert single node
/// db.exec_mut(&QueryBuilder::insert().nodes().count(1).query()).unwrap();
///
/// // Insert single node as a transaction
/// db.transaction_mut(|t| -> Result<(), QueryError> { t.exec_mut(&QueryBuilder::insert().nodes().count(1).query())?; Ok(()) }).unwrap();
///
/// // Select single database element with id 1
/// db.exec(&QueryBuilder::select().ids(1).query()).unwrap();
///
/// // Select single database element with id 1 as a transaction
/// db.transaction(|t| -> Result<(), QueryError> { t.exec(&QueryBuilder::select().ids(1).query())?; Ok(()) }).unwrap();
///
/// // Search the database starting at element 1
/// db.exec(&QueryBuilder::search().from(1).query()).unwrap();
/// ```
/// # Transactions
///
/// All queries are transactions. Explicit transactions take closures that are passed
/// the transaction object to record & execute queries. You cannot explicitly commit
/// nor rollback transactions. To commit a transaction simply return `Ok` from the
/// transaction closure. Conversely to rollback a transaction return `Err`. Nested
/// transactions are not allowed.
///
/// # Multithreading
///
/// The `agdb` is multithreading enabled. It is recommended to use `Arc<RwLock>`:
///
/// ```
/// use std::sync::{Arc, RwLock};
/// use agdb::Db;
///
/// let db = Arc::new(RwLock::new(Db::new("db.agdb").unwrap()));
/// db.read().unwrap(); //for a read lock allowing Db::exec() and Db::transaction()
/// db.write().unwrap(); //for a write lock allowing additionally Db::exec_mut() and Db::transaction_mut()
/// ```
/// Using the database in the multi-threaded environment is then the same as in a single
/// threaded application (minus the locking). Nevertheless while Rust does prevent
/// race conditions you still need to be on a lookout for potential deadlocks. This is
/// one of the reasons why nested transactions are not supported by the `agdb`.
///
/// Akin to the Rust borrow checker rules the `agdb` can handle unlimited number
/// of concurrent reads (transactional or regular) but only single write operation
/// at any one time. For that reason the transactions are not database states or objects
/// but rather a function taking a closure executing the queries in an attempt to limit
/// their scope as much as possible (and therefore the duration of the [exclusive] lock).
///
/// # Storage
///
/// The `agdb` is using a single database file to store all of its data. Additionally
/// a single shadow file with a `.` prefix of the main database file name is used as
/// a write ahead log (WAL). On drop of the [`Db`] object the WAL is processed and removed
/// aborting any unfinished transactions. Furthermore the database data is defragmented.
///
/// On load, if the WAL file is present (e.g. due to a crash), it will be processed
/// restoring any consistent state that existed before the crash. Data is only
/// written to the main file if the reverse operation has been committed to the
/// WAL file. The WAL is then purged on commit of a transaction (all queries are
/// transactional even if the transaction is not explicitly used).
pub struct DbImpl<Store: StorageData> {
    storage: Storage<Store>,
    graph: DbGraph<Store>,
    aliases: DbIndexedMap<String, DbId, Store>,
    indexes: DbIndexes<Store>,
    values: MultiMapStorage<DbId, DbKeyValue, Store>,
    undo_stack: Vec<Command>,
}

/// The default implementation of the database using memory mapped file (full ACID) with
/// write ahead log. If your data set exceeds available (or reasonable amount) of memory
/// consider using [`DbFile`] instead. You can load the file created with [`DbFile`] and
/// vice versa.
pub type Db = DbImpl<FileStorageMemoryMapped>;

/// A convenience alias for the [`Transaction`] type for the default [`Db`].
pub type DbTransaction<'a> = Transaction<'a, FileStorageMemoryMapped>;

/// A convenience alias for the [`TransactionMut`] type for the default [`Db`].
pub type DbTransactionMut<'a> = TransactionMut<'a, FileStorageMemoryMapped>;

/// The file based implementation of the database (full ACID) with write ahead logging and
/// but minimum memory footprint but slower than the default [`Db`]. You can load the file
/// created with [`Db`] and vice versa.
pub type DbFile = DbImpl<FileStorage>;

/// A convenience alias for the [`Transaction`] type for the default [`DbFile`].
pub type DbFileTransaction<'a> = Transaction<'a, FileStorage>;

/// A convenience alias for the [`TransactionMut`] type for the default [`DbFile`].
pub type DbFileTransactionMut<'a> = TransactionMut<'a, FileStorage>;

/// The purely in-memory implementation of the database. It has no persistence but offers
/// unmatched performance
pub type DbMemory = DbImpl<MemoryStorage>;

/// A convenience alias for the [`Transaction`] type for the default [`DbMemory`].
pub type DbMemoryTransaction<'a> = Transaction<'a, MemoryStorage>;

/// A convenience alias for the [`TransactionMut`] type for the default [`DbMemory`].
pub type DbMemoryTransactionMut<'a> = TransactionMut<'a, MemoryStorage>;

impl<Store: StorageData> std::fmt::Debug for DbImpl<Store> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("agdb::Db").finish_non_exhaustive()
    }
}

impl<Store: StorageData> DbImpl<Store> {
    /// Tries to create or load `filename` file as `Db` object.
    pub fn new(filename: &str) -> Result<Self, DbError> {
        match Self::try_new(filename) {
            Ok(db) => Ok(db),
            Err(error) => {
                let mut db_error = DbError::from("Failed to create database");
                db_error.cause = Some(Box::new(error));
                Err(db_error)
            }
        }
    }

    /// Flushes the underlying file and copies it
    /// to `filename` path. Consider calling `optimize_storage()`
    /// prior to this function to reduce the size of the storage
    /// file. If speed is of the essence you may omit that operation
    /// at expense of the file size.
    pub fn backup(&self, filename: &str) -> Result<(), DbError> {
        self.storage.backup(filename)
    }

    /// Copies the database to `filename` path.  Consider calling
    /// `optimize_storage()` prior to this function to reduce the
    /// size of the storage file to be copied.
    pub fn copy(&self, filename: &str) -> Result<Self, DbError> {
        let storage = self.storage.copy(filename)?;
        let index = storage.value::<DbStorageIndex>(StorageIndex(1))?;

        let graph = DbGraph::from_storage(&storage, index.graph)?;
        let aliases = DbIndexedMap::from_storage(&storage, index.aliases)?;
        let indexes = DbIndexes::from_storage(&storage, index.indexes)?;
        let values = MultiMapStorage::from_storage(&storage, index.values)?;

        Ok(Self {
            storage,
            graph,
            aliases,
            indexes,
            values,
            undo_stack: vec![],
        })
    }

    /// Executes immutable query:
    ///
    /// - Select elements
    /// - Select values
    /// - Select keys
    /// - Select key count
    /// - Select aliases
    /// - Select all aliases
    /// - Search
    ///
    /// It runs the query as a transaction and returns either the result or
    /// error describing what went wrong (e.g. query error, logic error, data
    /// error etc.).
    pub fn exec<T: Query>(&self, query: &T) -> Result<QueryResult, QueryError> {
        self.transaction(|transaction| transaction.exec(query))
    }

    /// Executes mutable query:
    ///
    /// - Insert nodes
    /// - Insert edges
    /// - Insert aliases
    /// - Insert values
    /// - Remove elements
    /// - Remove aliases
    /// - Remove values
    ///
    /// It runs the query as a transaction and returns either the result or
    /// error describing what went wrong (e.g. query error, logic error, data
    /// error etc.).
    pub fn exec_mut<T: QueryMut>(&mut self, query: &T) -> Result<QueryResult, QueryError> {
        self.transaction_mut(|transaction| transaction.exec_mut(query))
    }

    /// Returns the filename that was used to
    /// construct the database.
    pub fn filename(&self) -> &str {
        self.storage.name()
    }

    /// Reclaims no longer used segments of the database file by packing all
    /// used storage segments together. This operation is done automatically
    /// when the database goes out of scope. In long running programs it might
    /// be desired to perform the storage file optimization without fully shutting
    /// down.
    pub fn optimize_storage(&mut self) -> Result<(), DbError> {
        self.storage.shrink_to_fit()
    }

    /// Changes the name of the database changing also the names of the files
    /// (if the storage is file based).
    pub fn rename(&mut self, filename: &str) -> Result<(), DbError> {
        self.storage.rename(filename)
    }

    /// Executes immutable transaction. The transaction is running a closure `f`
    /// that will receive `&Transaction` object to run `exec` queries as if run
    /// on the main database object. You shall specify the return type `T`
    /// (can be `()`) and the error type `E` that must be constructible from the `QueryError`
    /// (`E` can be `QueryError`).
    ///
    /// Read transactions cannot be committed or rolled back but their main function is to ensure
    /// that the database data does not change during their duration. Through its generic
    /// parameters it also allows transforming the query results into a type `T`.
    pub fn transaction<T, E>(
        &self,
        mut f: impl FnMut(&Transaction<Store>) -> Result<T, E>,
    ) -> Result<T, E> {
        let transaction = Transaction::new(self);

        f(&transaction)
    }

    /// Executes mutable transaction. The transaction is running a closure `f`
    /// that will receive `&mut Transaction` to execute `exec` and `exec_mut` queries
    /// as if run on the main database object. You shall specify the return type `T`
    /// (can be `()`) and the error type `E` that must be constructible from the `QueryError`
    /// (`E` can be `QueryError`).
    ///
    /// Write transactions are committed if the closure returns `Ok` and rolled back if
    /// the closure returns `Err`. If the code panics and the program exits the write
    /// ahead log (WAL) makes sure the data in the main database file is restored to a
    /// consistent state prior to the transaction.
    ///
    /// Typical use case for a write transaction is to insert nodes and edges together.
    /// When not using a transaction you could end up only with nodes being inserted.
    ///
    /// Through its generic parameters the transaction also allows transforming the query
    /// results into a type `T`.
    pub fn transaction_mut<T, E: From<QueryError>>(
        &mut self,
        mut f: impl FnMut(&mut TransactionMut<Store>) -> Result<T, E>,
    ) -> Result<T, E> {
        let mut transaction = TransactionMut::new(&mut *self);
        let result = f(&mut transaction);

        if result.is_ok() {
            transaction.commit()?;
        } else {
            transaction.rollback()?;
        }

        result
    }

    /// Returns the database size in bytes. Depending on the underlying storage
    /// the physical size in hardware might be higher. For example for in-memory
    /// storage this function reports used storage but actually allocated memory
    /// will likely be higher (expecting database growth in order to prevent too
    /// frequent allocations). For file based storages this number will be accurate
    /// but the actually used space on disk will be higher up to the next file system
    /// block size.
    pub fn size(&self) -> u64 {
        self.storage.len()
    }

    pub(crate) fn commit(&mut self) -> Result<(), QueryError> {
        self.undo_stack.clear();
        Ok(())
    }

    pub(crate) fn rollback(&mut self) -> Result<(), QueryError> {
        let mut undo_stack = vec![];
        std::mem::swap(&mut undo_stack, &mut self.undo_stack);

        for command in undo_stack.iter().rev() {
            match command {
                Command::InsertAlias { id, alias } => {
                    self.aliases.insert(&mut self.storage, alias, id)?
                }
                Command::InsertEdge { from, to } => self
                    .graph
                    .insert_edge(&mut self.storage, *from, *to)
                    .map(|_| ())?,
                Command::InsertIndex { key } => {
                    self.indexes.insert(&mut self.storage, key.clone())?;
                }
                Command::InsertToIndex { key, value, id } => self
                    .indexes
                    .index_mut(key)
                    .expect("index not found during rollback")
                    .ids_mut()
                    .insert(&mut self.storage, value, id)?,
                Command::InsertKeyValue { id, key_value } => {
                    if let Some(index) = self.indexes.index_mut(&key_value.key) {
                        index
                            .ids_mut()
                            .insert(&mut self.storage, &key_value.value, id)?;
                    }
                    self.values.insert(&mut self.storage, id, key_value)?
                }
                Command::InsertNode => self.graph.insert_node(&mut self.storage).map(|_| ())?,
                Command::RemoveAlias { alias } => {
                    self.aliases.remove_key(&mut self.storage, alias)?
                }
                Command::RemoveEdge { index } => {
                    self.graph.remove_edge(&mut self.storage, *index)?
                }
                Command::RemoveIndex { key } => self.indexes.remove(&mut self.storage, key)?,
                Command::RemoveKeyValue { id, key_value } => {
                    if let Some(index) = self.indexes.index_mut(&key_value.key) {
                        index
                            .ids_mut()
                            .remove_value(&mut self.storage, &key_value.value, id)?;
                    }
                    self.values.remove_value(&mut self.storage, id, key_value)?
                }
                Command::RemoveNode { index } => {
                    self.graph.remove_node(&mut self.storage, *index)?
                }
                Command::ReplaceKeyValue { id, key_value } => {
                    let old = self
                        .values
                        .insert_or_replace(
                            &mut self.storage,
                            id,
                            |v| v.key == key_value.key,
                            key_value,
                        )?
                        .expect("old value not found during rollback");

                    if let Some(index) = self.indexes.index_mut(&old.key) {
                        index
                            .ids_mut()
                            .remove_value(&mut self.storage, &old.value, id)?;
                        index
                            .ids_mut()
                            .insert(&mut self.storage, &key_value.value, id)?;
                    }

                    return Ok(());
                }
            }
        }

        Ok(())
    }

    pub(crate) fn alias(&self, db_id: DbId) -> Result<String, QueryError> {
        self.aliases
            .key(&self.storage, &db_id)?
            .ok_or(QueryError::from(format!("Id '{}' not found", db_id.0)))
    }

    pub(crate) fn aliases(&self) -> Vec<(String, DbId)> {
        self.aliases.iter(&self.storage).collect()
    }

    pub(crate) fn db_id(&self, query_id: &QueryId) -> Result<DbId, QueryError> {
        match query_id {
            QueryId::Id(id) => Ok(DbId(self.graph_index(id.0)?.0)),
            QueryId::Alias(alias) => Ok(self
                .aliases
                .value(&self.storage, alias)?
                .ok_or(QueryError::from(format!("Alias '{alias}' not found")))?),
        }
    }

    pub(crate) fn edge_count(&self, db_id: DbId, from: bool, to: bool) -> Result<u64, QueryError> {
        let index = self.graph_index(db_id.0)?;

        if let Some(node) = self.graph.node(&self.storage, index) {
            return Ok(match (from, to) {
                (true, true) => node.edge_count(),
                (true, false) => node.edge_count_from(),
                (false, true) => node.edge_count_to(),
                (false, false) => 0,
            });
        }

        Ok(0)
    }

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn from_id(&self, id: DbId) -> Option<DbId> {
        if id.0 < 0 {
            Some(DbId(
                self.graph.edge_from(&self.storage, GraphIndex(id.0)).0,
            ))
        } else {
            None
        }
    }

    pub(crate) fn to_id(&self, id: DbId) -> Option<DbId> {
        if id.0 < 0 {
            Some(DbId(self.graph.edge_to(&self.storage, GraphIndex(id.0)).0))
        } else {
            None
        }
    }

    pub(crate) fn indexes(&self) -> Vec<DbKeyValue> {
        self.indexes
            .indexes()
            .iter()
            .map(|index| DbKeyValue {
                key: index.key().clone(),
                value: index.ids().len().into(),
            })
            .collect()
    }

    pub(crate) fn insert_alias(&mut self, db_id: DbId, alias: &String) -> Result<(), DbError> {
        if let Some(old_alias) = self.aliases.key(&self.storage, &db_id)? {
            self.undo_stack.push(Command::InsertAlias {
                id: db_id,
                alias: old_alias.clone(),
            });
            self.aliases.remove_key(&mut self.storage, &old_alias)?;
            self.aliases.remove_key(&mut self.storage, &old_alias)?;
        }

        self.undo_stack.push(Command::RemoveAlias {
            alias: alias.clone(),
        });
        self.aliases.insert(&mut self.storage, alias, &db_id)
    }

    pub(crate) fn insert_edge(&mut self, from: DbId, to: DbId) -> Result<DbId, QueryError> {
        let index =
            self.graph
                .insert_edge(&mut self.storage, GraphIndex(from.0), GraphIndex(to.0))?;
        self.undo_stack.push(Command::RemoveEdge { index });

        Ok(DbId(index.0))
    }

    pub(crate) fn insert_index(&mut self, key: &DbValue) -> Result<u64, QueryError> {
        if self.indexes.index(key).is_some() {
            return Err(QueryError::from(format!("Index '{key}' already exists")));
        }

        let values = self
            .values
            .iter(&self.storage)
            .filter_map(|(id, kv)| {
                if kv.key == *key {
                    Some((id, kv.value))
                } else {
                    None
                }
            })
            .collect::<Vec<(DbId, DbValue)>>();

        self.undo_stack
            .push(Command::RemoveIndex { key: key.clone() });
        let index = self.indexes.insert(&mut self.storage, key.clone())?;

        for (id, value) in values {
            index.ids_mut().insert(&mut self.storage, &value, &id)?;
        }

        Ok(index.ids().len())
    }

    pub(crate) fn insert_new_alias(
        &mut self,
        db_id: DbId,
        alias: &String,
    ) -> Result<(), QueryError> {
        if let Some(id) = self.aliases.value(&self.storage, alias)? {
            return Err(QueryError::from(format!(
                "Alias '{alias}' already exists ({})",
                id.0
            )));
        }

        self.undo_stack.push(Command::RemoveAlias {
            alias: alias.clone(),
        });
        self.aliases.insert(&mut self.storage, alias, &db_id)?;

        Ok(())
    }

    pub(crate) fn insert_node(&mut self) -> Result<DbId, DbError> {
        let index = self.graph.insert_node(&mut self.storage)?;
        self.undo_stack.push(Command::RemoveNode { index });

        Ok(DbId(index.0))
    }

    pub(crate) fn insert_key_value(
        &mut self,
        db_id: DbId,
        key_value: &DbKeyValue,
    ) -> Result<(), QueryError> {
        if let Some(index) = self.indexes.index_mut(&key_value.key) {
            index
                .ids_mut()
                .insert(&mut self.storage, &key_value.value, &db_id)?;
        }

        self.undo_stack.push(Command::RemoveKeyValue {
            id: db_id,
            key_value: key_value.clone(),
        });
        self.values.insert(&mut self.storage, &db_id, key_value)?;
        Ok(())
    }

    pub(crate) fn insert_or_replace_key_value(
        &mut self,
        db_id: DbId,
        key_value: &DbKeyValue,
    ) -> Result<(), QueryError> {
        if let Some(old) = self.values.insert_or_replace(
            &mut self.storage,
            &db_id,
            |kv| kv.key == key_value.key,
            key_value,
        )? {
            if let Some(index) = self.indexes.index_mut(&old.key) {
                index
                    .ids_mut()
                    .remove_value(&mut self.storage, &old.value, &db_id)?;
                index
                    .ids_mut()
                    .insert(&mut self.storage, &key_value.value, &db_id)?;
            }

            self.undo_stack.push(Command::ReplaceKeyValue {
                id: db_id,
                key_value: old,
            });
        } else {
            if let Some(index) = self.indexes.index_mut(&key_value.key) {
                index
                    .ids_mut()
                    .insert(&mut self.storage, &key_value.value, &db_id)?;
            }

            self.undo_stack.push(Command::RemoveKeyValue {
                id: db_id,
                key_value: key_value.clone(),
            });
        }

        Ok(())
    }

    pub(crate) fn keys(&self, db_id: DbId) -> Result<Vec<DbKeyValue>, DbError> {
        Ok(self
            .values
            .iter_key(&self.storage, &db_id)
            .map(|kv| (kv.1.key, DbValue::default()).into())
            .collect())
    }

    pub(crate) fn key_count(&self, db_id: DbId) -> Result<u64, DbError> {
        self.values.values_count(&self.storage, &db_id)
    }

    pub(crate) fn remove(&mut self, query_id: &QueryId) -> Result<bool, QueryError> {
        match query_id {
            QueryId::Id(db_id) => self.remove_id(*db_id),
            QueryId::Alias(alias) => {
                if let Some(db_id) = self.aliases.value(&self.storage, alias)? {
                    self.remove_node(db_id, GraphIndex(db_id.0), Some(alias.clone()))?;
                    self.remove_all_values(db_id)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    pub(crate) fn remove_id(&mut self, db_id: DbId) -> Result<bool, QueryError> {
        if let Ok(graph_index) = self.graph_index(db_id.0) {
            if graph_index.is_node() {
                self.remove_node(db_id, graph_index, self.aliases.key(&self.storage, &db_id)?)?;
            } else {
                self.remove_edge(graph_index)?;
            }
            self.remove_all_values(db_id)?;
            return Ok(true);
        }

        Ok(false)
    }

    pub(crate) fn remove_alias(&mut self, alias: &String) -> Result<bool, DbError> {
        if let Some(id) = self.aliases.value(&self.storage, alias)? {
            self.undo_stack.push(Command::InsertAlias {
                id,
                alias: alias.clone(),
            });
            self.aliases.remove_key(&mut self.storage, alias)?;
            self.aliases.remove_key(&mut self.storage, alias)?;

            return Ok(true);
        }

        Ok(false)
    }

    pub(crate) fn remove_index(&mut self, key: &DbValue) -> Result<u64, QueryError> {
        let mut count = None;

        if let Some(index) = self.indexes.index(key) {
            for (value, id) in index.ids().iter(&self.storage) {
                self.undo_stack.push(Command::InsertToIndex {
                    key: key.clone(),
                    value: value.clone(),
                    id,
                });
            }

            count = Some(index.ids().len());
        }

        if let Some(count) = count {
            self.undo_stack
                .push(Command::InsertIndex { key: key.clone() });
            self.indexes.remove(&mut self.storage, key)?;
            Ok(count)
        } else {
            Ok(0)
        }
    }

    pub(crate) fn search_index(
        &self,
        key: &DbValue,
        value: &DbValue,
    ) -> Result<Vec<DbId>, QueryError> {
        if let Some(index) = self.indexes.index(key) {
            Ok(index.ids().values(&self.storage, value)?)
        } else {
            Err(QueryError::from(format!("Index '{key}' not found")))
        }
    }

    pub(crate) fn search_from(
        &self,
        from: DbId,
        algorithm: SearchQueryAlgorithm,
        limit: u64,
        offset: u64,
        conditions: &Vec<QueryCondition>,
    ) -> Result<Vec<DbId>, QueryError> {
        let search = GraphSearch::from((&self.graph, &self.storage));

        let indexes = match (limit, offset) {
            (0, 0) => match algorithm {
                SearchQueryAlgorithm::BreadthFirst => search.breadth_first_search(
                    GraphIndex(from.0),
                    DefaultHandler::new(self, conditions),
                )?,
                _ => search.depth_first_search(
                    GraphIndex(from.0),
                    DefaultHandler::new(self, conditions),
                )?,
            },

            (_, 0) => match algorithm {
                SearchQueryAlgorithm::BreadthFirst => search.breadth_first_search(
                    GraphIndex(from.0),
                    LimitHandler::new(limit, self, conditions),
                )?,
                _ => search.depth_first_search(
                    GraphIndex(from.0),
                    LimitHandler::new(limit, self, conditions),
                )?,
            },

            (0, _) => match algorithm {
                SearchQueryAlgorithm::BreadthFirst => search.breadth_first_search(
                    GraphIndex(from.0),
                    OffsetHandler::new(offset, self, conditions),
                )?,
                _ => search.depth_first_search(
                    GraphIndex(from.0),
                    OffsetHandler::new(offset, self, conditions),
                )?,
            },

            (_, _) => match algorithm {
                SearchQueryAlgorithm::BreadthFirst => search.breadth_first_search(
                    GraphIndex(from.0),
                    LimitOffsetHandler::new(limit, offset, self, conditions),
                )?,
                _ => search.depth_first_search(
                    GraphIndex(from.0),
                    LimitOffsetHandler::new(limit, offset, self, conditions),
                )?,
            },
        };

        Ok(indexes.iter().map(|index| DbId(index.0)).collect())
    }

    pub(crate) fn search_to(
        &self,
        to: DbId,
        algorithm: SearchQueryAlgorithm,
        limit: u64,
        offset: u64,
        conditions: &Vec<QueryCondition>,
    ) -> Result<Vec<DbId>, QueryError> {
        let search = GraphSearch::from((&self.graph, &self.storage));

        let indexes = match (limit, offset) {
            (0, 0) => match algorithm {
                SearchQueryAlgorithm::BreadthFirst => search.breadth_first_search_reverse(
                    GraphIndex(to.0),
                    DefaultHandler::new(self, conditions),
                )?,
                _ => search.depth_first_search_reverse(
                    GraphIndex(to.0),
                    DefaultHandler::new(self, conditions),
                )?,
            },

            (_, 0) => match algorithm {
                SearchQueryAlgorithm::BreadthFirst => search.breadth_first_search_reverse(
                    GraphIndex(to.0),
                    LimitHandler::new(limit, self, conditions),
                )?,
                _ => search.depth_first_search_reverse(
                    GraphIndex(to.0),
                    LimitHandler::new(limit, self, conditions),
                )?,
            },

            (0, _) => match algorithm {
                SearchQueryAlgorithm::BreadthFirst => search.breadth_first_search_reverse(
                    GraphIndex(to.0),
                    OffsetHandler::new(offset, self, conditions),
                )?,
                _ => search.depth_first_search_reverse(
                    GraphIndex(to.0),
                    OffsetHandler::new(offset, self, conditions),
                )?,
            },

            (_, _) => match algorithm {
                SearchQueryAlgorithm::BreadthFirst => search.breadth_first_search_reverse(
                    GraphIndex(to.0),
                    LimitOffsetHandler::new(limit, offset, self, conditions),
                )?,
                _ => search.depth_first_search_reverse(
                    GraphIndex(to.0),
                    LimitOffsetHandler::new(limit, offset, self, conditions),
                )?,
            },
        };

        Ok(indexes.iter().map(|index| DbId(index.0)).collect())
    }

    pub(crate) fn search_from_to(
        &self,
        from: DbId,
        to: DbId,
        conditions: &Vec<QueryCondition>,
    ) -> Result<Vec<DbId>, QueryError> {
        Ok(GraphSearch::from((&self.graph, &self.storage))
            .path(
                GraphIndex(from.0),
                GraphIndex(to.0),
                PathHandler::new(self, conditions),
            )?
            .iter()
            .map(|index| DbId(index.0))
            .collect())
    }

    pub(crate) fn values(&self, db_id: DbId) -> Result<Vec<DbKeyValue>, DbError> {
        self.values.values(&self.storage, &db_id)
    }

    pub(crate) fn values_by_keys(
        &self,
        db_id: DbId,
        keys: &[DbValue],
    ) -> Result<Vec<DbKeyValue>, DbError> {
        Ok(self
            .values
            .iter_key(&self.storage, &db_id)
            .filter(|kv| keys.contains(&kv.1.key))
            .map(|kv| kv.1)
            .collect())
    }

    fn graph_index(&self, id: i64) -> Result<GraphIndex, QueryError> {
        match id.cmp(&0) {
            std::cmp::Ordering::Less => {
                if self.graph.edge(&self.storage, GraphIndex(id)).is_some() {
                    return Ok(GraphIndex(id));
                }
            }
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => {
                if self.graph.node(&self.storage, GraphIndex(id)).is_some() {
                    return Ok(GraphIndex(id));
                }
            }
        }

        Err(QueryError::from(format!("Id '{id}' not found")))
    }

    fn node_edges(
        &self,
        graph_index: GraphIndex,
    ) -> Result<Vec<(GraphIndex, GraphIndex, GraphIndex)>, DbError> {
        let mut edges = vec![];
        let node = self
            .graph
            .node(&self.storage, graph_index)
            .ok_or(DbError::from("Data integrity corrupted"))?;

        for edge in node.edge_iter_from() {
            edges.push((edge.index(), edge.index_from(), edge.index_to()));
        }

        for edge in node.edge_iter_to() {
            let from = edge.index_from();
            if from != graph_index {
                edges.push((edge.index(), from, edge.index_to()));
            }
        }

        Ok(edges)
    }

    fn remove_edge(&mut self, graph_index: GraphIndex) -> Result<(), DbError> {
        let (from, to) = {
            let edge = self
                .graph
                .edge(&self.storage, graph_index)
                .ok_or(DbError::from("Graph integrity corrupted"))?;
            (edge.index_from(), edge.index_to())
        };

        self.graph.remove_edge(&mut self.storage, graph_index)?;
        self.undo_stack.push(Command::InsertEdge { from, to });
        Ok(())
    }

    fn remove_node(
        &mut self,
        db_id: DbId,
        graph_index: GraphIndex,
        alias: Option<String>,
    ) -> Result<(), DbError> {
        if let Some(alias) = alias {
            self.undo_stack.push(Command::InsertAlias {
                alias: alias.clone(),
                id: db_id,
            });
            self.aliases.remove_key(&mut self.storage, &alias)?;
            self.aliases.remove_key(&mut self.storage, &alias)?;
        }

        for edge in self.node_edges(graph_index)? {
            self.graph.remove_edge(&mut self.storage, edge.0)?;
            self.undo_stack.push(Command::InsertEdge {
                from: edge.1,
                to: edge.2,
            });
            self.remove_all_values(DbId(edge.0 .0))?;
        }

        self.graph.remove_node(&mut self.storage, graph_index)?;
        self.undo_stack.push(Command::InsertNode);
        Ok(())
    }

    pub(crate) fn remove_keys(&mut self, db_id: DbId, keys: &[DbValue]) -> Result<i64, QueryError> {
        let mut result = 0;

        for key_value in self.values.values(&self.storage, &db_id)? {
            if keys.contains(&key_value.key) {
                if let Some(index) = self.indexes.index_mut(&key_value.key) {
                    index
                        .ids_mut()
                        .remove_value(&mut self.storage, &key_value.value, &db_id)?;
                }
                self.values
                    .remove_value(&mut self.storage, &db_id, &key_value)?;
                self.undo_stack.push(Command::InsertKeyValue {
                    id: db_id,
                    key_value,
                });
                result -= 1;
            }
        }

        Ok(result)
    }

    fn remove_all_values(&mut self, db_id: DbId) -> Result<(), DbError> {
        for key_value in self.values.values(&self.storage, &db_id)? {
            if let Some(index) = self.indexes.index_mut(&key_value.key) {
                index
                    .ids_mut()
                    .remove_value(&mut self.storage, &key_value.value, &db_id)?;
            }

            self.undo_stack.push(Command::InsertKeyValue {
                id: db_id,
                key_value,
            });
        }

        self.values.remove_key(&mut self.storage, &db_id)?;

        Ok(())
    }

    fn try_new(filename: &str) -> Result<Self, DbError> {
        let mut storage = Storage::new(filename)?;
        let graph_storage;
        let aliases_storage;
        let indexes_storage;
        let values_storage;
        let len = storage.len();
        let index = storage.value::<DbStorageIndex>(StorageIndex(1));

        if let Ok(index) = index {
            graph_storage = DbGraph::from_storage(&storage, index.graph)?;
            aliases_storage = DbIndexedMap::from_storage(&storage, index.aliases)?;
            indexes_storage = DbIndexes::from_storage(&storage, index.indexes)?;
            values_storage = MultiMapStorage::from_storage(&storage, index.values)?;
        } else if len == 0 {
            storage.insert(&DbStorageIndex::default())?;
            graph_storage = DbGraph::new(&mut storage)?;
            aliases_storage = DbIndexedMap::new(&mut storage)?;
            indexes_storage = DbIndexes::new(&mut storage)?;
            values_storage = MultiMapStorage::new(&mut storage)?;
            let db_storage_index = DbStorageIndex {
                graph: graph_storage.storage_index(),
                aliases: aliases_storage.storage_index(),
                indexes: indexes_storage.storage_index(),
                values: values_storage.storage_index(),
            };
            storage.insert_at(StorageIndex(1), 0, &db_storage_index)?;
        } else {
            return Err(DbError::from(format!(
                "File '{filename}' is not a valid database file and is not empty."
            )));
        }

        Ok(Self {
            storage,
            graph: graph_storage,
            aliases: aliases_storage,
            indexes: indexes_storage,
            values: values_storage,
            undo_stack: vec![],
        })
    }

    pub(crate) fn evaluate_condition(
        &self,
        index: GraphIndex,
        distance: u64,
        condition: &QueryConditionData,
    ) -> Result<SearchControl, DbError> {
        match condition {
            QueryConditionData::Distance(value) => Ok(value.compare_distance(distance)),
            QueryConditionData::Edge => Ok(SearchControl::Continue(index.is_edge())),
            QueryConditionData::EdgeCount(value) => Ok(SearchControl::Continue(
                if let Some(node) = self.graph.node(&self.storage, index) {
                    value.compare(node.edge_count())
                } else {
                    false
                },
            )),
            QueryConditionData::EdgeCountFrom(value) => Ok(SearchControl::Continue(
                if let Some(node) = self.graph.node(&self.storage, index) {
                    value.compare(node.edge_count_from())
                } else {
                    false
                },
            )),
            QueryConditionData::EdgeCountTo(value) => Ok(SearchControl::Continue(
                if let Some(node) = self.graph.node(&self.storage, index) {
                    value.compare(node.edge_count_to())
                } else {
                    false
                },
            )),
            QueryConditionData::Ids(values) => {
                Ok(SearchControl::Continue(values.iter().any(|id| {
                    index.0
                        == match id {
                            QueryId::Id(id) => id.0,
                            QueryId::Alias(alias) => {
                                self.aliases
                                    .value(&self.storage, alias)
                                    .unwrap_or_default()
                                    .unwrap_or_default()
                                    .0
                            }
                        }
                })))
            }
            QueryConditionData::KeyValue { key, value } => Ok(SearchControl::Continue(
                if let Some((_, kv)) = self
                    .values
                    .iter_key(&self.storage, &DbId(index.0))
                    .find(|(_, kv)| &kv.key == key)
                {
                    value.compare(&kv.value)
                } else {
                    false
                },
            )),
            QueryConditionData::Keys(values) => {
                let keys = self
                    .values
                    .iter_key(&self.storage, &DbId(index.0))
                    .map(|(_, kv)| kv.key)
                    .collect::<Vec<DbValue>>();
                Ok(SearchControl::Continue(
                    values.iter().all(|k| keys.contains(k)),
                ))
            }
            QueryConditionData::Node => Ok(SearchControl::Continue(index.is_node())),
            QueryConditionData::Where(conditions) => {
                self.evaluate_conditions(index, distance, conditions)
            }
        }
    }

    pub(crate) fn evaluate_conditions(
        &self,
        index: GraphIndex,
        distance: u64,
        conditions: &[QueryCondition],
    ) -> Result<SearchControl, DbError> {
        let mut result = SearchControl::Continue(true);

        for condition in conditions {
            let mut control = self.evaluate_condition(index, distance, &condition.data)?;

            match condition.modifier {
                QueryConditionModifier::Beyond => {
                    if control.is_true() {
                        control = control.and(SearchControl::Continue(true));
                    } else {
                        control = SearchControl::Stop(true);
                    }
                }
                QueryConditionModifier::Not => control.flip(),
                QueryConditionModifier::NotBeyond => {
                    if control.is_true() {
                        control = control.and(SearchControl::Stop(true));
                    } else {
                        control = SearchControl::Continue(true);
                    }
                }
                _ => {}
            };

            result = match condition.logic {
                QueryConditionLogic::And => result.and(control),
                QueryConditionLogic::Or => result.or(control),
            };
        }

        Ok(result)
    }
}

impl<Store: StorageData> Drop for DbImpl<Store> {
    fn drop(&mut self) {
        let _ = self.storage.shrink_to_fit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn db_storage_index_serialized_size() {
        assert_eq!(DbStorageIndex::default().serialized_size(), 40);
    }

    #[test]
    fn derived_from_debug() {
        let test_file = TestFile::new();
        let db = Db::new(test_file.file_name()).unwrap();
        format!("{:?}", db);
    }
}
