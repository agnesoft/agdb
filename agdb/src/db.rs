pub mod db_element;
pub mod db_error;
pub mod db_f64;
pub mod db_id;
pub mod db_index;
pub mod db_key_order;
pub mod db_key_value;
pub mod db_migration;
pub mod db_type;
pub mod db_value;

mod db_aliases;
mod db_search_handlers;
mod db_value_index;

use self::db_aliases::DbAliases;
use self::db_error::DbError;
use self::db_error::DbErrorType;
use self::db_migration::DbMigration;
use self::db_search_handlers::DefaultHandler;
use self::db_search_handlers::LimitHandler;
use self::db_search_handlers::LimitOffsetHandler;
use self::db_search_handlers::OffsetHandler;
use self::db_search_handlers::PathHandler;
use crate::Amend;
use crate::DbId;
use crate::DbKeyValue;
use crate::DbValue;
use crate::QueryResult;
use crate::SearchQueryAlgorithm;
use crate::StorageData;
use crate::Transaction;
use crate::TransactionMut;
use crate::command::Command;
use crate::db::db_index::DbIndexes;
use crate::db::db_key_value::DbKeyValues;
use crate::graph::DbGraph;
use crate::graph::GraphIndex;
use crate::graph_search::GraphSearch;
use crate::graph_search::SearchControl;
use crate::query::Query;
use crate::query::QueryMut;
use crate::query::query_condition::QueryCondition;
use crate::query::query_condition::QueryConditionData;
use crate::query::query_condition::QueryConditionLogic;
use crate::query::query_condition::QueryConditionModifier;
use crate::query::query_id::QueryId;
use crate::storage::Storage;
use crate::storage::StorageConfig;
use crate::storage::StorageIndex;
use crate::storage::SyncMode;
use crate::storage::any_storage::AnyStorage;
use crate::storage::file_storage::FileStorage;
use crate::storage::file_storage_memory_mapped::FileStorageMemoryMapped;
use crate::storage::memory_storage::MemoryStorage;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use crate::utilities::serialize::u64_at;

/// Version of the database format. A database of an older version is
/// migrated when opened and a newer one is refused.
pub const DB_VERSION: u64 = 2;

#[derive(Default)]
struct DbStorageIndex {
    version: u64,
    graph: StorageIndex,
    aliases: StorageIndex,
    indexes: StorageIndex,
    values: StorageIndex,
}

impl Serialize for DbStorageIndex {
    fn serialize(&self) -> Vec<u8> {
        [
            self.version.serialize(),
            self.graph.serialize(),
            self.aliases.serialize(),
            self.indexes.serialize(),
            self.values.serialize(),
        ]
        .concat()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        if bytes.len() < Self::serialized_size_static() as usize {
            return Err(DbError::db(
                DbErrorType::NotEnoughData,
                format!(
                    "DbStorageIndex deserialization error: got {} bytes, expected at least {} bytes",
                    bytes.len(),
                    Self::serialized_size_static(),
                ),
            ));
        }

        let field = |i: usize| u64_at(bytes, i);

        Ok(Self {
            version: field(0),
            graph: StorageIndex(field(1)),
            aliases: StorageIndex(field(2)),
            indexes: StorageIndex(field(3)),
            values: StorageIndex(field(4)),
        })
    }

    fn serialized_size(&self) -> u64 {
        Self::serialized_size_static()
    }
}

impl SerializeStatic for DbStorageIndex {
    fn serialized_size_static() -> u64 {
        i64::serialized_size_static() * 5
    }
}

/// An instance of the `agdb` database. To create a database:
///
/// ```
/// # let _test_file = agdb::test_utilities::test_file::TestFile::from("db1.agdb");
/// use agdb::Db;
///
/// let mut db = Db::new("db1.agdb").unwrap();
/// ```
///
/// This will try to create or load the database at the given file path.
/// If the file does not exist a new database will be initialized creating
/// the given file. If the file does exist the database will try to load
/// it and memory map the data.
///
/// These are the available variants of the database to choose from:
///
/// - [`Db`]: \[default] File based and memory mapped database.
/// - [`DbFile`]: File based only (no memory mapping).
/// - [`DbMemory`]: In-memory database only.
/// - [`DbAny`]: Database variant that can be any of the other variants.
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
/// # let _test_file = agdb::test_utilities::test_file::TestFile::from("db2.agdb");
/// use agdb::{Db, QueryBuilder, DbError};
///
/// let mut db = Db::new("db2.agdb").unwrap();
///
/// // Insert single node
/// db.exec_mut(QueryBuilder::insert().nodes().count(1).query()).unwrap();
///
/// // Insert single node as a transaction
/// db.transaction_mut(|t| -> Result<(), DbError> { t.exec_mut(QueryBuilder::insert().nodes().count(1).query())?; Ok(()) }).unwrap();
///
/// // Select single database element with id 1
/// db.exec(QueryBuilder::select().ids(1).query()).unwrap();
///
/// // Select single database element with id 1 as a transaction
/// db.transaction(|t| -> Result<(), DbError> { t.exec(QueryBuilder::select().ids(1).query())?; Ok(()) }).unwrap();
///
/// // Search the database starting at element 1
/// db.exec(QueryBuilder::search().from(1).query()).unwrap();
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
/// # let _test_file = agdb::test_utilities::test_file::TestFile::from("db3.agdb");
/// use std::sync::{Arc, RwLock};
/// use agdb::Db;
///
/// let db = Arc::new(RwLock::new(Db::new("db3.agdb").unwrap()));
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
    aliases: DbAliases<Store>,
    indexes: DbIndexes<Store>,
    values: DbKeyValues<Store>,
    undo_stack: Vec<Command>,
    poisoned: bool,
    migration: Option<DbMigration>,
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

/// The file based implementation of the database (full ACID) with write ahead logging,
/// minimum memory footprint, but slower than the default [`Db`]. You can load the file
/// created with [`Db`] and vice versa.
pub type DbFile = DbImpl<FileStorage>;

/// A convenience alias for the [`Transaction`] type for [`DbFile`].
pub type DbFileTransaction<'a> = Transaction<'a, FileStorage>;

/// A convenience alias for the [`TransactionMut`] type for [`DbFile`].
pub type DbFileTransactionMut<'a> = TransactionMut<'a, FileStorage>;

/// The purely in-memory implementation of the database. It has no persistence but offers
/// unmatched performance.
pub type DbMemory = DbImpl<MemoryStorage>;

/// A convenience alias for the [`Transaction`] type for [`DbMemory`].
pub type DbMemoryTransaction<'a> = Transaction<'a, MemoryStorage>;

/// A convenience alias for the [`TransactionMut`] type for [`DbMemory`].
pub type DbMemoryTransactionMut<'a> = TransactionMut<'a, MemoryStorage>;

/// A convenience alias for a Db type that can use any implemented storage (mapped, memory or file).
pub type DbAny = DbImpl<AnyStorage>;

/// A convenience alias for the [`Transaction`] type for [`DbAny`].
pub type DbAnyTransaction<'a> = Transaction<'a, AnyStorage>;

/// A convenience alias for the [`TransactionMut`] type for [`DbAny`].
pub type DbAnyTransactionMut<'a> = TransactionMut<'a, AnyStorage>;

impl<Store: StorageData> std::fmt::Debug for DbImpl<Store> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("agdb::Db").finish_non_exhaustive()
    }
}

impl<Store: StorageData> DbImpl<Store> {
    /// Tries to create or load `filename` file as `Db` object. For in-memory storage
    /// this will either load the data from file once (if present) or create an empty database.
    /// If used with the `DbAny` variant the database will be of variant `Db` (memory mapped). Use
    /// `DbAny::new_*()` to construct the other variants.
    pub fn new(filename: &str) -> Result<Self, DbError> {
        match Self::try_new(filename) {
            Ok(db) => Ok(db),
            Err(error) => {
                let mut db_error = DbError::db(
                    DbErrorType::DbCreate,
                    format!("Failed to create database: {filename}"),
                );
                db_error.cause = Some(Box::new(error));
                Err(db_error)
            }
        }
    }

    /// Tries to create a new database with the given `data` store.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::{DbMemory, MemoryStorage, StorageData};
    ///
    /// let mut db = DbMemory::with_data(MemoryStorage::new("test").unwrap()).unwrap();
    /// ```
    pub fn with_data(data: Store) -> Result<Self, DbError> {
        Self::try_new_with_storage(Storage::with_data(data)?)
    }

    /// Flushes the underlying file and copies it
    /// to `filename` path. Consider calling `optimize_storage()`
    /// prior to this function to reduce the size of the storage
    /// file. If speed is of the essence you may omit that operation
    /// at expense of the file size. For memory based storage this will
    /// dump the internal buffer to the `filename` which can be used to
    /// restore back the database by `DbMemory::new()`.
    pub fn backup(&self, filename: &str) -> Result<(), DbError> {
        self.storage.backup(filename)
    }

    /// Copies the database to `filename` path.  Consider calling
    /// `optimize_storage()` prior to this function to reduce the
    /// size of the storage file to be copied.
    pub fn copy(&self, filename: &str) -> Result<Self, DbError> {
        let storage = self.storage.copy(filename)?;
        let (graph, aliases, indexes, values) = Self::load_data_structures(&storage)?;

        Ok(Self {
            storage,
            graph,
            aliases,
            indexes,
            values,
            undo_stack: vec![],
            poisoned: false,
            migration: None,
        })
    }

    /// Executes immutable query:
    ///
    /// - Select elements
    /// - Select values
    /// - Select keys
    /// - Select key count
    /// - Select edge count
    /// - Select aliases
    /// - Select all aliases
    /// - Select indexes
    /// - Select node count
    /// - Search
    ///
    /// It runs the query as a transaction and returns either the result or
    /// error describing what went wrong (e.g. query error, logic error, data
    /// error etc.).
    pub fn exec<T: Query>(&self, query: T) -> Result<QueryResult, DbError> {
        self.transaction(|transaction| transaction.exec(query))
            .map_err(self.poisoned())
    }

    /// Executes mutable query:
    ///
    /// - Insert nodes
    /// - Insert edges
    /// - Insert aliases
    /// - Insert values
    /// - Insert index
    /// - Remove elements
    /// - Remove aliases
    /// - Remove values
    /// - Remove index
    ///
    /// It runs the query as a transaction and returns either the result or
    /// error describing what went wrong (e.g. query error, logic error, data
    /// error etc.).
    pub fn exec_mut<T: QueryMut>(&mut self, query: T) -> Result<QueryResult, DbError> {
        self.transaction_mut(|transaction| transaction.exec_mut(query))
            .map_err(self.poisoned())
    }

    /// Returns the filename that was used to
    /// construct the database.
    pub fn filename(&self) -> &str {
        self.storage.name()
    }

    /// Returns the conversion of the database to the current version
    /// done when it was opened, `None` if it already was of the current
    /// version. The conversion is permanent so the older versions of
    /// `agdb` can no longer open the database.
    pub fn migration(&self) -> Option<&DbMigration> {
        self.migration.as_ref()
    }

    /// Reclaims no longer used segments of the database file by packing all
    /// used storage segments together. This operation is done automatically
    /// when the database goes out of scope. In long running programs it might
    /// be desired to perform the storage file optimization without fully shutting
    /// down.
    pub fn optimize_storage(&mut self) -> Result<(), DbError> {
        self.storage.optimize_storage().map_err(|e| self.recover(e))
    }

    /// Changes the name of the database changing also the names of the files
    /// (if the storage is file based).
    pub fn rename(&mut self, filename: &str) -> Result<(), DbError> {
        self.storage.rename(filename)
    }

    /// Executes immutable transaction. The transaction is running a closure `f`
    /// that will receive `&Transaction` object to run `exec` queries as if run
    /// on the main database object. You shall specify the return type `T`
    /// (can be `()`) and the error type `E` that must be constructible from the `DbError`
    /// (`E` can be `DbError`).
    ///
    /// Read transactions cannot be committed or rolled back but their main function is to ensure
    /// that the database data does not change during their duration. Through its generic
    /// parameters it also allows transforming the query results into a type `T`.
    pub fn transaction<T, E>(
        &self,
        f: impl FnOnce(&Transaction<Store>) -> Result<T, E>,
    ) -> Result<T, E> {
        let transaction = Transaction::new(self);
        f(&transaction)
    }

    /// Executes mutable transaction. The transaction is running a closure `f`
    /// that will receive `&mut Transaction` to execute `exec` and `exec_mut` queries
    /// as if run on the main database object. You shall specify the return type `T`
    /// (can be `()`) and the error type `E` that must be constructible from the `DbError`
    /// (`E` can be `DbError`).
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
    pub fn transaction_mut<T, E: From<DbError>>(
        &mut self,
        f: impl FnOnce(&mut TransactionMut<Store>) -> Result<T, E>,
    ) -> Result<T, E> {
        let storage_transaction = self.storage.transaction();
        let mut transaction = TransactionMut::new(&mut *self);
        let result = f(&mut transaction);

        let commit_result = if result.is_ok() {
            transaction.commit()
        } else {
            transaction.rollback()
        };

        match commit_result {
            Ok(()) => {
                if let Err(e) = self.storage.commit(storage_transaction) {
                    return Err(E::from(self.recover(e)));
                }
            }
            Err(rollback_err) => {
                let e = self.recover(rollback_err);

                if self.poisoned {
                    return Err(E::from(e));
                }

                return result;
            }
        }

        result
    }

    /// Ensures all buffered data reaches durable storage. Call before
    /// reading raw database files for snapshots or backups. This is
    /// independent of `sync_mode` and always issues the sync syscall.
    pub fn sync(&mut self) -> Result<(), DbError> {
        self.storage.sync()
    }

    /// Returns the [`StorageConfig`] of this database.
    pub fn storage_config(&self) -> StorageConfig {
        StorageConfig {
            sync_mode: self.storage.sync_mode(),
            overlay_limit: self.storage.overlay_limit(),
        }
    }

    /// Sets the [`StorageConfig`] of this database, see [`SyncMode`] and
    /// [`StorageData::set_overlay_limit()`].
    pub fn set_storage_config(&mut self, config: StorageConfig) {
        self.storage.set_sync_mode(config.sync_mode);
        self.storage.set_overlay_limit(config.overlay_limit);
    }

    /// Returns the current [`SyncMode`] for this database.
    pub fn sync_mode(&self) -> SyncMode {
        self.storage.sync_mode()
    }

    /// Sets the sync mode for this database. See [`SyncMode`] for details.
    pub fn set_sync_mode(&mut self, mode: SyncMode) {
        self.storage.set_sync_mode(mode);
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

    /// Shrinks all the allocated internal storage buffers to the current size of the data
    /// and optimizes the storage. This may further reduce the size of the database
    /// both on disk and in memory at the cost of requiring more allocations upon inserting
    /// more data. Best to use only when the database is used for read-only operations after
    /// initial data population.
    pub fn shrink_to_fit(&mut self) -> Result<(), DbError> {
        self.graph.shrink_to_fit(&mut self.storage)?;
        self.aliases.shrink_to_fit(&mut self.storage)?;
        self.indexes.shrink_to_fit(&mut self.storage)?;
        self.values.shrink_to_fit(&mut self.storage)?;
        self.optimize_storage()
    }

    fn poisoned(&self) -> impl FnOnce(DbError) -> DbError + '_ {
        |e| {
            if self.poisoned {
                DbError::db(
                    DbErrorType::Poisoned,
                    "database poisoned & reload failed - drop and reopen required",
                )
                .caused_by(e)
            } else {
                e
            }
        }
    }

    #[allow(clippy::type_complexity)]
    fn load_data_structures(
        storage: &Storage<Store>,
    ) -> Result<
        (
            DbGraph<Store>,
            DbAliases<Store>,
            DbIndexes<Store>,
            DbKeyValues<Store>,
        ),
        DbError,
    > {
        let index = storage.value::<DbStorageIndex>(StorageIndex(1))?;

        if index.version != DB_VERSION {
            return Err(DbError::db(
                DbErrorType::NotAllowed,
                format!(
                    "Database version '{}' is not the current version '{DB_VERSION}'",
                    index.version
                ),
            ));
        }

        Ok((
            DbGraph::from_storage(storage, index.graph)?,
            DbAliases::from_storage(storage, index.aliases)?,
            DbIndexes::from_storage(storage, index.indexes)?,
            DbKeyValues::from_storage(storage, index.values)?,
        ))
    }

    fn reload_from_storage(&mut self) -> Result<(), DbError> {
        let (graph, aliases, indexes, values) = Self::load_data_structures(&self.storage)?;
        self.graph = graph;
        self.aliases = aliases;
        self.indexes = indexes;
        self.values = values;
        self.undo_stack.clear();
        Ok(())
    }

    fn recover(&mut self, cause: DbError) -> DbError {
        match self
            .storage
            .recover()
            .and_then(|_| self.reload_from_storage())
        {
            Ok(()) => cause,
            Err(e) => {
                self.poison();
                (self.poisoned())(e.caused_by(cause))
            }
        }
    }

    fn poison(&mut self) {
        self.poisoned = true;
        self.storage.wipe_records();
        self.undo_stack.clear();
    }

    pub(crate) fn commit(&mut self) -> Result<(), DbError> {
        self.undo_stack.clear();
        Ok(())
    }

    pub(crate) fn rollback(&mut self) -> Result<(), DbError> {
        let mut undo_stack = vec![];
        std::mem::swap(&mut undo_stack, &mut self.undo_stack);

        for command in undo_stack.iter().rev() {
            match command {
                Command::InsertAlias { id, alias } => {
                    self.aliases
                        .insert(&mut self.storage, &mut self.graph, *id, alias)?
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
                    self.values
                        .insert_value(&mut self.storage, id.as_index(), key_value)?
                }
                Command::InsertNode => self.graph.insert_node(&mut self.storage).map(|_| ())?,
                Command::RemoveAlias { id } => {
                    self.aliases
                        .remove(&mut self.storage, &mut self.graph, *id)?;
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
                    self.values
                        .remove_value(&mut self.storage, id.as_index(), &key_value.key)?
                }
                Command::RemoveNode { index } => {
                    self.graph.remove_node(&mut self.storage, *index)?
                }
                Command::ReplaceKeyValue { id, key_value } => {
                    let old = self
                        .values
                        .insert_or_replace(&mut self.storage, id.as_index(), key_value)?
                        .expect("old value not found during rollback");

                    if let Some(index) = self.indexes.index_mut(&old.key) {
                        index
                            .ids_mut()
                            .remove_value(&mut self.storage, &old.value, id)?;
                        index
                            .ids_mut()
                            .insert(&mut self.storage, &key_value.value, id)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) fn alias(&self, db_id: DbId) -> Result<String, DbError> {
        self.aliases
            .alias(&self.storage, &self.graph, db_id)?
            .ok_or(DbError::db(
                DbErrorType::NotFound,
                format!("Id '{}' not found", db_id.0),
            ))
    }

    pub(crate) fn aliases(&self) -> Result<Vec<(String, DbId)>, DbError> {
        self.aliases.aliases(&self.storage, &self.graph)
    }

    pub(crate) fn db_id(&self, query_id: &QueryId) -> Result<DbId, DbError> {
        match query_id {
            QueryId::Id(id) => Ok(DbId(self.graph_index(id.0)?.0)),
            QueryId::Alias(alias) => Ok(self
                .aliases
                .id(&self.storage, &self.graph, alias)?
                .ok_or(DbError::db(
                    DbErrorType::NotFound,
                    format!("Alias '{alias}' not found"),
                ))?),
        }
    }

    pub(crate) fn all_edge_count(&self) -> Result<u64, DbError> {
        self.graph.edge_count(&self.storage)
    }

    pub(crate) fn edge_count(&self, db_id: DbId, from: bool, to: bool) -> Result<u64, DbError> {
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
    pub(crate) fn from_id(&self, id: DbId) -> Result<DbId, DbError> {
        if id.0 < 0 {
            Ok(DbId(
                self.graph.edge_from(&self.storage, GraphIndex(id.0)).0,
            ))
        } else {
            Ok(DbId(
                self.graph
                    .first_edge_from(&self.storage, GraphIndex(id.0))?
                    .0,
            ))
        }
    }

    pub(crate) fn to_id(&self, id: DbId) -> Result<DbId, DbError> {
        if id.0 < 0 {
            Ok(DbId(self.graph.edge_to(&self.storage, GraphIndex(id.0)).0))
        } else {
            Ok(DbId(
                self.graph.first_edge_to(&self.storage, GraphIndex(id.0))?.0,
            ))
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

    /// Sets the `alias` of the node `db_id` replacing its current alias
    /// and taking the alias from the node that had it.
    pub(crate) fn insert_alias(&mut self, db_id: DbId, alias: &String) -> Result<(), DbError> {
        if alias.is_empty() {
            return Err(DbError::query(
                DbErrorType::NotAllowed,
                "Empty alias is not allowed",
            ));
        }

        if db_id.0 < 0 {
            return Err(DbError::query(
                DbErrorType::NotAllowed,
                format!("Edges cannot have aliases: '{alias}' on '{}'", db_id.0),
            ));
        }

        if let Some(old_alias) = self.aliases.alias(&self.storage, &self.graph, db_id)? {
            if old_alias == *alias {
                return Ok(());
            }

            self.aliases
                .remove(&mut self.storage, &mut self.graph, db_id)?;
            self.undo_stack.push(Command::InsertAlias {
                id: db_id,
                alias: old_alias,
            });
        }

        if let Some(owner) = self.aliases.id(&self.storage, &self.graph, alias)? {
            self.aliases
                .remove(&mut self.storage, &mut self.graph, owner)?;
            self.undo_stack.push(Command::InsertAlias {
                id: owner,
                alias: alias.clone(),
            });
        }

        self.undo_stack.push(Command::RemoveAlias { id: db_id });
        self.aliases
            .insert(&mut self.storage, &mut self.graph, db_id, alias)
    }

    pub(crate) fn insert_edge(&mut self, from: DbId, to: DbId) -> Result<DbId, DbError> {
        let index =
            self.graph
                .insert_edge(&mut self.storage, GraphIndex(from.0), GraphIndex(to.0))?;
        self.undo_stack.push(Command::RemoveEdge { index });

        Ok(DbId(index.0))
    }

    pub(crate) fn insert_index(&mut self, key: &DbValue) -> Result<u64, DbError> {
        if self.indexes.index(key).is_some() {
            return Err(DbError::db(
                DbErrorType::NotAllowed,
                format!("Index '{key}' already exists"),
            ));
        }

        self.undo_stack
            .push(Command::RemoveIndex { key: key.clone() });

        let index = self.indexes.insert(&mut self.storage, key.clone())?;

        for i in 1..self.values.len() {
            let kvs = self.values.values(&self.storage, i)?;
            let db_id = if self
                .graph
                .node(&self.storage, GraphIndex(i as i64))
                .is_some()
            {
                DbId(i as i64)
            } else {
                DbId(-(i as i64))
            };

            for kv in kvs {
                if kv.key == *key {
                    index
                        .ids_mut()
                        .insert(&mut self.storage, &kv.value, &db_id)?;
                }
            }
        }

        Ok(index.ids().len())
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
    ) -> Result<(), DbError> {
        if let Some(index) = self.indexes.index_mut(&key_value.key) {
            index
                .ids_mut()
                .insert(&mut self.storage, &key_value.value, &db_id)?;
        }

        self.undo_stack.push(Command::RemoveKeyValue {
            id: db_id,
            key_value: key_value.clone(),
        });
        self.values
            .insert_value(&mut self.storage, db_id.as_index(), key_value)?;
        Ok(())
    }

    pub(crate) fn insert_or_replace_key_value(
        &mut self,
        db_id: DbId,
        key_value: &DbKeyValue,
    ) -> Result<(), DbError> {
        if let Some(old) =
            self.values
                .insert_or_replace(&mut self.storage, db_id.as_index(), key_value)?
        {
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

    pub(crate) fn amend_key_value(
        &mut self,
        db_id: DbId,
        key_value: &DbKeyValue,
        amend: &Amend,
    ) -> Result<bool, DbError> {
        let existing = self
            .values
            .value(&self.storage, db_id.as_index(), &key_value.key)?;

        match existing {
            Some(existing_value) => {
                let merged_value = match amend {
                    Amend::Add => existing_value.amend_add(&key_value.value)?,
                    Amend::Remove => existing_value.amend_remove(&key_value.value)?,
                    Amend::AddBitwise(op) => {
                        existing_value.amend_bitwise(&key_value.value, op, true)?
                    }
                    Amend::RemoveBitwise(op) => {
                        existing_value.amend_bitwise(&key_value.value, op, false)?
                    }
                    Amend::None => unreachable!(),
                };

                let merged_kv = DbKeyValue {
                    key: key_value.key.clone(),
                    value: merged_value,
                };
                self.insert_or_replace_key_value(db_id, &merged_kv)?;
                Ok(true)
            }
            None => match amend {
                Amend::Add | Amend::AddBitwise(_) => {
                    self.insert_or_replace_key_value(db_id, key_value)?;
                    Ok(true)
                }
                Amend::Remove | Amend::RemoveBitwise(_) => Ok(false),
                Amend::None => unreachable!(),
            },
        }
    }

    pub(crate) fn keys(&self, db_id: DbId) -> Result<Vec<DbValue>, DbError> {
        self.values.keys(&self.storage, db_id.as_index())
    }

    pub(crate) fn key_count(&self, db_id: DbId) -> Result<u64, DbError> {
        self.values.key_count(&self.storage, db_id.as_index())
    }

    pub(crate) fn node_count(&self) -> Result<u64, DbError> {
        self.graph.node_count(&self.storage)
    }

    pub(crate) fn remove(&mut self, query_id: &QueryId) -> Result<bool, DbError> {
        match query_id {
            QueryId::Id(db_id) => self.remove_id(*db_id),
            QueryId::Alias(alias) => {
                if let Some(db_id) = self.aliases.id(&self.storage, &self.graph, alias)? {
                    self.remove_node(db_id, GraphIndex(db_id.0), Some(alias.clone()))?;
                    self.remove_all_values(db_id)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    pub(crate) fn remove_id(&mut self, db_id: DbId) -> Result<bool, DbError> {
        if let Ok(graph_index) = self.graph_index(db_id.0) {
            if graph_index.is_node() {
                let alias = self.aliases.alias(&self.storage, &self.graph, db_id)?;
                self.remove_node(db_id, graph_index, alias)?;
            } else {
                self.remove_edge(graph_index)?;
            }
            self.remove_all_values(db_id)?;
            return Ok(true);
        }

        Ok(false)
    }

    pub(crate) fn remove_alias(&mut self, alias: &str) -> Result<bool, DbError> {
        if let Some(id) = self.aliases.id(&self.storage, &self.graph, alias)? {
            self.aliases
                .remove(&mut self.storage, &mut self.graph, id)?;
            self.undo_stack.push(Command::InsertAlias {
                id,
                alias: alias.to_string(),
            });

            return Ok(true);
        }

        Ok(false)
    }

    pub(crate) fn remove_index(&mut self, key: &DbValue) -> Result<u64, DbError> {
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

    pub(crate) fn reserve_key_value_capacity(
        &mut self,
        db_id: DbId,
        additional: u64,
    ) -> Result<(), DbError> {
        self.values
            .reserve_capacity(&mut self.storage, db_id.as_index(), additional)
    }

    pub(crate) fn search_index(
        &self,
        key: &DbValue,
        value: &DbValue,
    ) -> Result<Vec<DbId>, DbError> {
        if let Some(index) = self.indexes.index(key) {
            Ok(index.ids().values(&self.storage, value)?)
        } else {
            Err(DbError::db(
                DbErrorType::NotFound,
                format!("Index '{key}' not found"),
            ))
        }
    }

    pub(crate) fn search_from(
        &self,
        from: DbId,
        algorithm: SearchQueryAlgorithm,
        limit: u64,
        offset: u64,
        conditions: &Vec<QueryCondition>,
    ) -> Result<Vec<DbId>, DbError> {
        let search = GraphSearch::from((&self.graph, &self.storage));

        let indexes = match (limit, offset) {
            (0, 0) => match algorithm {
                SearchQueryAlgorithm::BreadthFirst => search.breadth_first_search(
                    GraphIndex(from.0),
                    DefaultHandler::new(self, conditions),
                )?,
                SearchQueryAlgorithm::DepthFirst => search.depth_first_search(
                    GraphIndex(from.0),
                    DefaultHandler::new(self, conditions),
                )?,
                _ => search.elements(DefaultHandler::new(self, conditions))?,
            },

            (_, 0) => match algorithm {
                SearchQueryAlgorithm::BreadthFirst => search.breadth_first_search(
                    GraphIndex(from.0),
                    LimitHandler::new(limit, self, conditions),
                )?,
                SearchQueryAlgorithm::DepthFirst => search.depth_first_search(
                    GraphIndex(from.0),
                    LimitHandler::new(limit, self, conditions),
                )?,
                _ => search.elements(LimitHandler::new(limit, self, conditions))?,
            },

            (0, _) => match algorithm {
                SearchQueryAlgorithm::BreadthFirst => search.breadth_first_search(
                    GraphIndex(from.0),
                    OffsetHandler::new(offset, self, conditions),
                )?,
                SearchQueryAlgorithm::DepthFirst => search.depth_first_search(
                    GraphIndex(from.0),
                    OffsetHandler::new(offset, self, conditions),
                )?,
                _ => search.elements(OffsetHandler::new(offset, self, conditions))?,
            },

            (_, _) => match algorithm {
                SearchQueryAlgorithm::BreadthFirst => search.breadth_first_search(
                    GraphIndex(from.0),
                    LimitOffsetHandler::new(limit, offset, self, conditions),
                )?,
                SearchQueryAlgorithm::DepthFirst => search.depth_first_search(
                    GraphIndex(from.0),
                    LimitOffsetHandler::new(limit, offset, self, conditions),
                )?,
                _ => search.elements(LimitOffsetHandler::new(limit, offset, self, conditions))?,
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
    ) -> Result<Vec<DbId>, DbError> {
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
    ) -> Result<Vec<DbId>, DbError> {
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

    pub(crate) fn search_to_from(
        &self,
        from: DbId,
        to: DbId,
        conditions: &Vec<QueryCondition>,
    ) -> Result<Vec<DbId>, DbError> {
        Ok(GraphSearch::from((&self.graph, &self.storage))
            .path_reverse(
                GraphIndex(from.0),
                GraphIndex(to.0),
                PathHandler::new(self, conditions),
            )?
            .iter()
            .map(|index| DbId(index.0))
            .collect())
    }

    pub(crate) fn values(&self, db_id: DbId) -> Result<Vec<DbKeyValue>, DbError> {
        self.values.values(&self.storage, db_id.as_index())
    }

    pub(crate) fn values_by_keys(
        &self,
        db_id: DbId,
        keys: &[DbValue],
    ) -> Result<Vec<DbKeyValue>, DbError> {
        self.values
            .values_by_keys(&self.storage, db_id.as_index(), keys)
    }

    fn graph_index(&self, id: i64) -> Result<GraphIndex, DbError> {
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

        Err(DbError::db(
            DbErrorType::NotFound,
            format!("Id '{id}' not found"),
        ))
    }

    fn node_edges(
        &self,
        graph_index: GraphIndex,
    ) -> Result<Vec<(GraphIndex, GraphIndex, GraphIndex)>, DbError> {
        let mut edges = vec![];
        let node = self
            .graph
            .node(&self.storage, graph_index)
            .ok_or(DbError::db(
                DbErrorType::NotFound,
                format!("Node {} not found", graph_index.0),
            ))?;

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
                .ok_or(DbError::db(
                    DbErrorType::NotFound,
                    format!("Edge {} not found", graph_index.0),
                ))?;
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
            self.aliases
                .remove(&mut self.storage, &mut self.graph, db_id)?;
            self.undo_stack
                .push(Command::InsertAlias { alias, id: db_id });
        }

        for edge in self.node_edges(graph_index)? {
            self.graph.remove_edge(&mut self.storage, edge.0)?;
            self.undo_stack.push(Command::InsertEdge {
                from: edge.1,
                to: edge.2,
            });
            self.remove_all_values(DbId(edge.0.0))?;
        }

        self.graph.remove_node(&mut self.storage, graph_index)?;
        self.undo_stack.push(Command::InsertNode);
        Ok(())
    }

    pub(crate) fn remove_keys(&mut self, db_id: DbId, keys: &[DbValue]) -> Result<u64, DbError> {
        let mut result = 0;

        for key_value in self.values.values(&self.storage, db_id.as_index())? {
            if keys.contains(&key_value.key) {
                if let Some(index) = self.indexes.index_mut(&key_value.key) {
                    index
                        .ids_mut()
                        .remove_value(&mut self.storage, &key_value.value, &db_id)?;
                }
                self.values
                    .remove_value(&mut self.storage, db_id.as_index(), &key_value.key)?;
                self.undo_stack.push(Command::InsertKeyValue {
                    id: db_id,
                    key_value,
                });
                result += 1;
            }
        }

        Ok(result)
    }

    fn remove_all_values(&mut self, db_id: DbId) -> Result<(), DbError> {
        for key_value in self.values.values(&self.storage, db_id.as_index())? {
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

        self.values.remove(&mut self.storage, db_id.as_index())?;

        Ok(())
    }

    fn try_new_with_storage(mut storage: Storage<Store>) -> Result<Self, DbError> {
        let graph;
        let aliases;
        let indexes;
        let values;
        let mut migration = None;

        if storage.value_size(StorageIndex(1)).is_err() {
            let id = storage.transaction();
            storage.insert(&DbStorageIndex::default())?;
            graph = DbGraph::new(&mut storage)?;
            aliases = DbAliases::new(&mut storage)?;
            indexes = DbIndexes::new(&mut storage)?;
            values = DbKeyValues::new(&mut storage)?;
            let db_storage_index = DbStorageIndex {
                version: DB_VERSION,
                graph: graph.storage_index(),
                aliases: aliases.storage_index(),
                indexes: indexes.storage_index(),
                values: values.storage_index(),
            };
            storage.insert_at(StorageIndex(1), 0, &db_storage_index)?;
            storage.set_version(DB_VERSION)?;
            storage.commit(id)?;
        } else {
            if storage.version() < DB_VERSION {
                migration = Some(db_migration::migrate(&mut storage)?);
            }

            (graph, aliases, indexes, values) = Self::load_data_structures(&storage)?;
        }

        Ok(Self {
            storage,
            graph,
            aliases,
            indexes,
            values,
            undo_stack: vec![],
            poisoned: false,
            migration,
        })
    }

    fn try_new(filename: &str) -> Result<Self, DbError> {
        Self::try_new_with_storage(Storage::new(filename)?)
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
                                    .id(&self.storage, &self.graph, alias)
                                    .unwrap_or_default()
                                    .unwrap_or_default()
                                    .0
                            }
                        }
                })))
            }
            QueryConditionData::KeyValue(kvc) => Ok(SearchControl::Continue(
                if let Some(value) = self.values.value(&self.storage, index.as_u64(), &kvc.key)? {
                    kvc.value.compare(&value)
                } else {
                    false
                },
            )),
            QueryConditionData::Keys(values) => {
                let keys = self.values.keys(&self.storage, index.as_u64())?;
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
                    if control.is_true() || distance == 0 {
                        control = SearchControl::Continue(result.is_true());
                    } else {
                        control = SearchControl::Stop(result.is_true());
                    }
                }
                QueryConditionModifier::Not => control.flip(),
                QueryConditionModifier::NotBeyond => {
                    if control.is_true() {
                        control = SearchControl::Stop(result.is_true());
                    } else {
                        control = SearchControl::Continue(result.is_true());
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
        if !self.poisoned {
            let _ = self.storage.optimize_storage();
        }
    }
}

impl DbAny {
    /// Creates a new DbAny instance using DbFile.
    pub fn new_file(filename: &str) -> Result<Self, DbError> {
        Self::try_new_any(filename, Self::try_new_file)
    }

    /// Creates a new DbAny instance using Db.
    pub fn new_mapped(filename: &str) -> Result<Self, DbError> {
        Self::try_new_any(filename, Self::try_new_mapped)
    }

    /// Creates a new DbAny instance using DbMemory.
    pub fn new_memory(filename: &str) -> Result<Self, DbError> {
        Self::try_new_any(filename, Self::try_new_memory)
    }

    fn try_new_any(
        filename: &str,
        init: fn(&str) -> Result<DbImpl<AnyStorage>, DbError>,
    ) -> Result<Self, DbError> {
        match init(filename) {
            Ok(db) => Ok(db),
            Err(error) => {
                let mut db_error = DbError::db(
                    DbErrorType::DbCreate,
                    format!("Failed to create database: {filename}"),
                );
                db_error.cause = Some(Box::new(error));
                Err(db_error)
            }
        }
    }

    fn try_new_memory(filename: &str) -> Result<DbImpl<AnyStorage>, DbError> {
        Self::try_new_with_storage(Storage::with_data(AnyStorage::Memory(MemoryStorage::new(
            filename,
        )?))?)
    }

    fn try_new_file(filename: &str) -> Result<DbImpl<AnyStorage>, DbError> {
        Self::try_new_with_storage(Storage::with_data(AnyStorage::File(FileStorage::new(
            filename,
        )?))?)
    }

    fn try_new_mapped(filename: &str) -> Result<DbImpl<AnyStorage>, DbError> {
        Self::try_new_with_storage(Storage::with_data(AnyStorage::MemoryMapped(
            FileStorageMemoryMapped::new(filename)?,
        ))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QueryBuilder;
    use crate::test_utilities::test_file::TestFile;
    use std::borrow::Cow;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn db_storage_index_serialized_size() {
        assert_eq!(DbStorageIndex::default().serialized_size(), 40);
    }

    #[test]
    fn db_storage_index_serialization() {
        let index = DbStorageIndex {
            version: 2,
            graph: StorageIndex(3),
            aliases: StorageIndex(4),
            indexes: StorageIndex(5),
            values: StorageIndex(6),
        };
        let deserialized = DbStorageIndex::deserialize(&index.serialize()).unwrap();
        assert_eq!(deserialized.version, 2);
        assert_eq!(deserialized.graph, StorageIndex(3));
        assert_eq!(deserialized.aliases, StorageIndex(4));
        assert_eq!(deserialized.indexes, StorageIndex(5));
        assert_eq!(deserialized.values, StorageIndex(6));
    }

    #[test]
    fn db_storage_index_short_bytes() {
        for len in [0, 17, 32, 39] {
            assert_eq!(
                DbStorageIndex::deserialize(&vec![0; len]).err().unwrap(),
                DbError::db(
                    DbErrorType::NotEnoughData,
                    format!(
                        "DbStorageIndex deserialization error: got {len} bytes, expected at least 40 bytes"
                    )
                )
            );
        }
    }

    #[test]
    fn load_data_structures_other_version_should_fail() {
        let mut storage = crate::collections::tests::storage();
        storage.insert(&DbStorageIndex::default()).unwrap();

        assert_eq!(
            DbImpl::load_data_structures(&storage).err(),
            Some(DbError::db(
                DbErrorType::NotAllowed,
                "Database version '0' is not the current version '2'"
            ))
        );
    }

    #[test]
    fn derived_from_debug() {
        let test_file = TestFile::new();
        let db = Db::new(test_file.file_name()).unwrap();
        let _ = format!("{db:?}");
    }

    #[derive(Debug)]
    struct FailableFileStorage {
        inner: FileStorage,
        fail_writes: Arc<AtomicBool>,
        fail_reads: Arc<AtomicBool>,
    }

    impl StorageData for FailableFileStorage {
        fn backup(&self, name: &str) -> Result<(), DbError> {
            self.inner.backup(name)
        }
        fn copy(&self, name: &str) -> Result<Self, DbError> {
            Ok(Self {
                inner: self.inner.copy(name)?,
                fail_writes: Arc::new(AtomicBool::new(false)),
                fail_reads: Arc::new(AtomicBool::new(false)),
            })
        }
        fn flush(&mut self) -> Result<(), DbError> {
            self.inner.flush()
        }
        fn len(&self) -> u64 {
            self.inner.len()
        }
        fn name(&self) -> &str {
            self.inner.name()
        }
        fn new(name: &str) -> Result<Self, DbError> {
            Ok(Self {
                inner: FileStorage::new(name)?,
                fail_writes: Arc::new(AtomicBool::new(false)),
                fail_reads: Arc::new(AtomicBool::new(false)),
            })
        }
        fn read(&self, pos: u64, value_len: u64) -> Result<Cow<'_, [u8]>, DbError> {
            if self.fail_reads.load(Ordering::Relaxed) {
                return Err(DbError::storage(
                    DbErrorType::NotAllowed,
                    "injected read failure",
                ));
            }
            self.inner.read(pos, value_len)
        }
        fn rename(&mut self, new_name: &str) -> Result<(), DbError> {
            self.inner.rename(new_name)
        }
        fn resize(&mut self, new_len: u64) -> Result<(), DbError> {
            if self.fail_writes.load(Ordering::Relaxed) {
                return Err(DbError::storage(
                    DbErrorType::NotAllowed,
                    "injected write failure",
                ));
            }
            self.inner.resize(new_len)
        }
        fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError> {
            if self.fail_writes.load(Ordering::Relaxed) {
                return Err(DbError::storage(
                    DbErrorType::NotAllowed,
                    "injected write failure",
                ));
            }
            self.inner.write(pos, bytes)
        }
        fn recover_from_wal(&mut self) -> Result<(), DbError> {
            self.inner.recover_from_wal()
        }
        fn set_sync_mode(&mut self, mode: SyncMode) {
            self.inner.set_sync_mode(mode);
        }
        fn sync_mode(&self) -> SyncMode {
            self.inner.sync_mode()
        }
        fn sync(&mut self) -> Result<(), DbError> {
            self.inner.sync()
        }
    }

    struct FailFlags {
        writes: Arc<AtomicBool>,
        reads: Arc<AtomicBool>,
    }

    fn failable_db(name: &str) -> (DbImpl<FailableFileStorage>, FailFlags) {
        let fail_writes = Arc::new(AtomicBool::new(false));
        let fail_reads = Arc::new(AtomicBool::new(false));
        let inner = FileStorage::new(name).unwrap();
        let data = FailableFileStorage {
            inner,
            fail_writes: fail_writes.clone(),
            fail_reads: fail_reads.clone(),
        };
        let storage = Storage::with_data(data).unwrap();
        let db = DbImpl::try_new_with_storage(storage).unwrap();
        (
            db,
            FailFlags {
                writes: fail_writes,
                reads: fail_reads,
            },
        )
    }

    #[test]
    fn rollback_failure_with_successful_recovery() {
        let test_file = TestFile::new();
        let (mut db, flags) = failable_db(test_file.file_name());

        db.exec_mut(
            QueryBuilder::insert()
                .nodes()
                .aliases("n")
                .values(vec![vec![("k", "original").into()]])
                .query(),
        )
        .unwrap();

        let fail = flags.writes.clone();
        let err = db
            .transaction_mut(|t| -> Result<(), DbError> {
                t.exec_mut(
                    QueryBuilder::insert()
                        .values(vec![vec![("k", "modified").into()]])
                        .ids("n")
                        .query(),
                )?;
                fail.store(true, Ordering::Relaxed);
                Err(DbError::db(DbErrorType::NotAllowed, "user error"))
            })
            .unwrap_err();

        assert_eq!(err.ty, DbErrorType::NotAllowed);
        assert_eq!(err.description, "user error");
        assert!(!db.poisoned);

        flags.writes.store(false, Ordering::Relaxed);
        let r = db.exec(QueryBuilder::select().ids("n").query()).unwrap();
        assert_eq!(r.elements[0].values, vec![("k", "original").into()]);
    }

    #[test]
    fn rollback_failure_with_failed_recovery_poisons() {
        let test_file = TestFile::new();
        let (mut db, flags) = failable_db(test_file.file_name());

        db.exec_mut(
            QueryBuilder::insert()
                .nodes()
                .aliases("n")
                .values(vec![vec![("k", "original").into()]])
                .query(),
        )
        .unwrap();

        let fail_w = flags.writes.clone();
        let fail_r = flags.reads.clone();
        let err = db
            .transaction_mut(|t| -> Result<(), DbError> {
                t.exec_mut(
                    QueryBuilder::insert()
                        .values(vec![vec![("k", "modified").into()]])
                        .ids("n")
                        .query(),
                )?;

                fail_w.store(true, Ordering::Relaxed);
                fail_r.store(true, Ordering::Relaxed);
                Err(DbError::db(DbErrorType::NotAllowed, "user error"))
            })
            .unwrap_err();

        assert_eq!(err.ty, DbErrorType::Poisoned);
        assert!(err.cause.is_some(), "poison error must chain the cause");
        assert!(db.poisoned);
    }

    #[test]
    fn poison_blocks_subsequent_operations() {
        let test_file = TestFile::new();
        let (mut db, _flags) = failable_db(test_file.file_name());

        db.exec_mut(
            QueryBuilder::insert()
                .nodes()
                .aliases("n")
                .values(vec![vec![("k", "original").into()]])
                .query(),
        )
        .unwrap();

        db.poison();

        let err = db
            .exec(QueryBuilder::select().ids("n").query())
            .unwrap_err();
        assert_eq!(err.ty, DbErrorType::Poisoned);

        let err = db
            .exec_mut(
                QueryBuilder::insert()
                    .values(vec![vec![("k", "v").into()]])
                    .ids("n")
                    .query(),
            )
            .unwrap_err();
        assert_eq!(err.ty, DbErrorType::Poisoned);
    }

    #[test]
    fn reopening_after_recovered_transaction_restores_data() {
        let test_file = TestFile::new();
        let db_path = test_file.file_name().clone();

        {
            let (mut db, flags) = failable_db(&db_path);
            db.exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .aliases("n")
                    .values(vec![vec![("k", "original").into()]])
                    .query(),
            )
            .unwrap();

            let fail_w = flags.writes.clone();

            let _ = db.transaction_mut(|t| -> Result<(), DbError> {
                t.exec_mut(
                    QueryBuilder::insert()
                        .values(vec![vec![("k", "modified").into()]])
                        .ids("n")
                        .query(),
                )?;
                fail_w.store(true, Ordering::Relaxed);
                Err(DbError::db(DbErrorType::NotAllowed, "user error"))
            });

            flags.writes.store(false, Ordering::Relaxed);
        }

        let (db, _) = failable_db(&db_path);
        let r = db.exec(QueryBuilder::select().ids("n").query()).unwrap();
        assert_eq!(r.elements[0].values, vec![("k", "original").into()]);
    }

    #[test]
    fn reopening_after_poisoned_transaction_restores_data() {
        let test_file = TestFile::new();
        let db_path = test_file.file_name().clone();

        {
            let (mut db, flags) = failable_db(&db_path);
            db.exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .aliases("n")
                    .values(vec![vec![("k", "original").into()]])
                    .query(),
            )
            .unwrap();

            let fail_w = flags.writes.clone();
            let fail_r = flags.reads.clone();
            let _ = db.transaction_mut(|t| -> Result<(), DbError> {
                t.exec_mut(
                    QueryBuilder::insert()
                        .values(vec![vec![("k", "modified").into()]])
                        .ids("n")
                        .query(),
                )?;
                fail_w.store(true, Ordering::Relaxed);
                fail_r.store(true, Ordering::Relaxed);
                Err(DbError::db(DbErrorType::NotAllowed, "user error"))
            });

            flags.writes.store(false, Ordering::Relaxed);
            flags.reads.store(false, Ordering::Relaxed);
        }

        let (db, _) = failable_db(&db_path);
        let r = db.exec(QueryBuilder::select().ids("n").query()).unwrap();
        assert_eq!(r.elements[0].values, vec![("k", "original").into()]);
    }

    mod db_impl {
        use super::*;
        use crate::storage::DEFAULT_OVERLAY_LIMIT;
        use crate::storage::file_storage::FileStorageImpl;
        use crate::storage::file_storage_memory_mapped::FileStorageMemoryMappedImpl;
        use agplatform::Platform;
        use agplatform::test_platform::TestPlatform;
        use agplatform::test_platform::test_fs::FsOp;
        use agplatform::test_platform::test_platform;
        use std::io::ErrorKind;

        const NAME: &str = "/db";
        const WAL: &str = "/.db";

        fn file(platform: &Arc<TestPlatform>) -> FileStorageImpl<TestPlatform> {
            FileStorageImpl::with_platform(platform.clone(), NAME).unwrap()
        }

        fn mapped(platform: &Arc<TestPlatform>) -> FileStorageMemoryMappedImpl<TestPlatform> {
            FileStorageMemoryMappedImpl::with_platform(platform.clone(), NAME).unwrap()
        }

        fn config(sync_mode: SyncMode, overlay_limit: u64) -> StorageConfig {
            StorageConfig {
                sync_mode,
                overlay_limit,
            }
        }

        fn open<D: StorageData>(data: D, config: StorageConfig) -> DbImpl<D> {
            let mut db = DbImpl::try_new_with_storage(Storage::with_data(data).unwrap()).unwrap();
            db.set_storage_config(config);
            db
        }

        fn inserted<D: StorageData>(data: D, config: StorageConfig) -> DbImpl<D> {
            let mut db = open(data, config);
            db.exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .aliases(["removed", "n"])
                    .values(vec![
                        vec![("k", vec![0_u64; 64]).into()],
                        vec![("k", "original").into()],
                    ])
                    .query(),
            )
            .unwrap();
            db.exec_mut(QueryBuilder::remove().ids("removed").query())
                .unwrap();
            db
        }

        fn modify<D: StorageData>(db: &mut DbImpl<D>) -> Result<QueryResult, DbError> {
            db.exec_mut(
                QueryBuilder::insert()
                    .values(vec![vec![("k", "modified").into()]])
                    .ids("n")
                    .query(),
            )
        }

        fn value<D: StorageData>(db: &DbImpl<D>) -> DbValue {
            db.exec(QueryBuilder::select().ids("n").query())
                .unwrap()
                .elements[0]
                .values[0]
                .value
                .clone()
        }

        fn kill(platform: &Arc<TestPlatform>) {
            platform.fs().sync();
            platform.fs().crash();
        }

        fn fail_once(platform: &Arc<TestPlatform>, op: FsOp, path: &str) {
            platform
                .fs()
                .set_io_errors(op, path, [Some(ErrorKind::Other), None]);
        }

        fn failed_commit<D: StorageData>(
            storage: fn(&Arc<TestPlatform>) -> D,
            config: StorageConfig,
            op: FsOp,
            path: &str,
        ) -> (bool, bool, DbValue, DbValue) {
            let platform = Arc::new(test_platform());
            let mut db = inserted(storage(&platform), config);
            fail_once(&platform, op, path);

            let failed = modify(&mut db).is_err();

            let current = value(&db);
            let poisoned = db.poisoned;
            kill(&platform);
            drop(db);
            (
                failed,
                poisoned,
                current,
                value(&open(storage(&platform), config)),
            )
        }

        mod transaction_mut {
            use super::*;

            #[test]
            fn transaction_mut_should_roll_back_failed_commit() {
                for (sync_mode, overlay_limit, op, path) in [
                    (SyncMode::None, DEFAULT_OVERLAY_LIMIT, FsOp::Write, WAL),
                    (SyncMode::None, DEFAULT_OVERLAY_LIMIT, FsOp::Write, NAME),
                    (SyncMode::Commit, DEFAULT_OVERLAY_LIMIT, FsOp::SyncData, WAL),
                    (
                        SyncMode::Commit,
                        DEFAULT_OVERLAY_LIMIT,
                        FsOp::SyncData,
                        NAME,
                    ),
                    (SyncMode::Commit, 0, FsOp::SyncData, NAME),
                    (
                        SyncMode::Strict,
                        DEFAULT_OVERLAY_LIMIT,
                        FsOp::SyncData,
                        NAME,
                    ),
                ] {
                    let config = config(sync_mode, overlay_limit);
                    let original = (true, false, "original".into(), "original".into());

                    assert_eq!(
                        failed_commit(file, config, op, path),
                        original,
                        "file {config:?} {op:?} {path}"
                    );
                    assert_eq!(
                        failed_commit(mapped, config, op, path),
                        original,
                        "mapped {config:?} {op:?} {path}"
                    );
                }
            }

            #[test]
            fn transaction_mut_should_commit_after_failed_commit() {
                let platform = Arc::new(test_platform());
                let mut db = inserted(file(&platform), StorageConfig::default());
                fail_once(&platform, FsOp::Write, NAME);
                assert!(modify(&mut db).is_err());

                modify(&mut db).unwrap();

                assert_eq!(value(&db), "modified".into());
                drop(db);
                assert_eq!(
                    value(&open(file(&platform), StorageConfig::default())),
                    "modified".into()
                );
            }

            #[test]
            fn transaction_mut_failed_recovery_after_failed_commit_should_poison() {
                let platform = Arc::new(test_platform());
                let mut db = inserted(file(&platform), StorageConfig::default());
                platform
                    .fs()
                    .set_io_error(FsOp::Write, NAME, ErrorKind::Other);

                let err = modify(&mut db).unwrap_err();

                assert_eq!(err.ty, DbErrorType::Poisoned);
                assert!(db.poisoned);
                platform.fs().clear_faults();
                drop(db);
                assert_eq!(
                    value(&open(file(&platform), StorageConfig::default())),
                    "original".into()
                );
            }
        }

        mod optimize_storage {
            use super::*;

            #[test]
            fn optimize_storage_failure_should_roll_back() {
                let platform = Arc::new(test_platform());
                let mut db = inserted(file(&platform), StorageConfig::default());
                let len = db.size();
                fail_once(&platform, FsOp::Write, NAME);

                assert!(db.optimize_storage().is_err());

                assert!(!db.poisoned);
                assert_eq!(db.size(), len);
                assert_eq!(value(&db), "original".into());
                db.optimize_storage().unwrap();
                assert!(db.size() < len);
            }
        }
    }
}
