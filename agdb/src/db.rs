pub mod db_element;
pub mod db_error;
pub mod db_f64;
pub mod db_id;
pub mod db_key_order;
pub mod db_key_value;
pub mod db_migration;
pub mod db_type;
pub mod db_value;

pub(crate) mod db_search_handlers;
pub(crate) mod db_value_index;
pub(crate) mod id_pool;
pub(crate) mod legacy;
pub(crate) mod rebuild;

use self::db_error::DbError;
use self::db_error::DbErrorType;
use self::db_migration::DbMigration;
use crate::QueryResult;
use crate::StorageData;
use crate::Transaction;
use crate::TransactionMut;
use crate::query::Query;
use crate::query::QueryMut;
use crate::storage::StorageConfig;
use crate::storage::SyncMode;
use crate::storage::any_storage::AnyStorage;
use crate::storage::concurrent_root::ConcurrentRoot;
use crate::storage::file_storage::FileStorage;
use crate::storage::file_storage_memory_mapped::FileStorageMemoryMapped;
use crate::storage::memory_storage::MemoryStorage;
use crate::storage::pages::Reader;
use crate::storage::root::Format;
use crate::storage::root::Layout;
use crate::storage::root::Root;

/// Version of the database format. A database of an older version is
/// migrated when opened and a newer one is refused.
pub const DB_VERSION: u64 = 2;

/// An instance of the `agdb` database. To create a database:
///
/// ```
/// # let _test_file = agdb::test_utilities::test_file::TestFile::from("db1.agdb");
/// use agdb::Db;
///
/// let db = Db::new("db1.agdb").unwrap();
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
/// let db = Db::new("db2.agdb").unwrap();
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
/// The `agdb` is multithreading enabled. All of its methods take `&self`, so it can be
/// shared between threads with an `Arc`:
///
/// ```
/// # let _test_file = agdb::test_utilities::test_file::TestFile::from("db3.agdb");
/// use std::sync::Arc;
/// use agdb::Db;
///
/// let db = Arc::new(Db::new("db3.agdb").unwrap());
/// ```
///
/// Reads (`exec()` and read-only [`transaction()`](Self::transaction)) run concurrently
/// on a consistent snapshot of the last commit and never wait for the writers.
///
/// Writes come in two kinds:
///
/// - [`transaction_mut()`](Self::transaction_mut) (and `exec_mut()`) writes exclusively:
///   one at a time, so it never conflicts.
/// - [`transaction()`](Self::transaction) with mutable queries runs concurrently with
///   other transactions and applies its writes when it commits (last writer wins, see
///   its documentation), failing with [`DbErrorType::Conflict`] only if an element it
///   writes was removed meanwhile.
///
/// The transactions are functions taking a closure rather than objects to keep them
/// as short as possible.
///
/// # Storage
///
/// The `agdb` is using a single database file to store all of its data. Additionally
/// a single shadow file with a `.` prefix of the main database file name is used as
/// a write ahead log (WAL). On drop of the [`Db`] object the WAL is processed and removed
/// aborting any unfinished transactions.
///
/// On load, if the WAL file is present (e.g. due to a crash), it will be processed
/// restoring any consistent state that existed before the crash. Data is only
/// written to the main file if the reverse operation has been committed to the
/// WAL file. The WAL is then purged on commit of a transaction (all queries are
/// transactional even if the transaction is not explicitly used).
pub struct DbImpl<Store: StorageData> {
    root: ConcurrentRoot<Store>,
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
        Store::new(filename)
            .and_then(Self::with_data)
            .map_err(|error| create_error(filename, error))
    }

    /// Tries to create a new database with the given `data` store.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::{DbMemory, MemoryStorage, StorageData};
    ///
    /// let db = DbMemory::with_data(MemoryStorage::new("test").unwrap()).unwrap();
    /// ```
    pub fn with_data(data: Store) -> Result<Self, DbError> {
        if let Format::Legacy(_) = Format::detect(&Reader::new(&data))? {
            let (root, migration) = rebuild::migrate(data)?;
            return Ok(Self {
                root: ConcurrentRoot::new(root),
                migration: Some(migration),
            });
        }

        let root = Root::open(data, Layout::default())?;
        Ok(Self {
            root: ConcurrentRoot::new(root),
            migration: None,
        })
    }

    /// Flushes the underlying file and copies it
    /// to `filename` path. Consider calling `optimize_storage()`
    /// prior to this function to reduce the size of the storage
    /// file. If speed is of the essence you may omit that operation
    /// at expense of the file size. For memory based storage this will
    /// dump the internal buffer to the `filename` which can be used to
    /// restore back the database by `DbMemory::new()`.
    ///
    /// It waits for the running write transaction (if any) and blocks
    /// the next ones until it is done, the read transactions keep running.
    /// Therefore it must not be called from within a transaction.
    pub fn backup(&self, filename: &str) -> Result<(), DbError> {
        self.root.with_storage(|storage| storage.backup(filename))
    }

    /// Copies the database to `filename` path.  Consider calling
    /// `optimize_storage()` prior to this function to reduce the
    /// size of the storage file to be copied.
    ///
    /// Like [`backup()`](Self::backup) it excludes the write transactions
    /// and must not be called from within a transaction.
    pub fn copy(&self, filename: &str) -> Result<Self, DbError> {
        let copy = self.root.with_storage(|storage| {
            let mut copy = storage.copy(filename)?;
            copy.set_sync_mode(storage.sync_mode());
            copy.set_overlay_limit(storage.overlay_limit());
            copy.set_memory_limit(storage.memory_limit());
            Ok::<_, DbError>(copy)
        })?;
        Self::with_data(copy)
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
    pub fn exec_mut<T: QueryMut>(&self, query: T) -> Result<QueryResult, DbError> {
        self.transaction_mut(|transaction| transaction.exec_mut(query))
    }

    /// Returns the filename that was used to
    /// construct the database.
    pub fn filename(&self) -> String {
        self.root.info().name
    }

    /// Returns the conversion of the database to the current version
    /// done when it was opened, `None` if it already was of the current
    /// version. The conversion is permanent so the older versions of
    /// `agdb` can no longer open the database.
    pub fn migration(&self) -> Option<&DbMigration> {
        self.migration.as_ref()
    }

    /// Reclaims no longer used segments of the database file by packing all
    /// used storage segments together. In long running programs it might
    /// be desired to perform the storage file optimization without fully shutting
    /// down.
    pub fn optimize_storage(&self) -> Result<(), DbError> {
        self.root.with_root_mut(|root| rebuild::optimize(root))
    }

    /// Changes the name of the database changing also the names of the files
    /// (if the storage is file based).
    pub fn rename(&self, filename: &str) -> Result<(), DbError> {
        self.root.with_storage_mut(|s| s.rename(filename))
    }

    /// Executes a concurrent transaction. The transaction is running a closure `f`
    /// that will receive `&Transaction` object to run `exec` and `exec_mut` queries
    /// as if run on the main database object. You shall specify the return type `T`
    /// (can be `()`) and the error type `E` that must be constructible from the `DbError`
    /// (`E` can be `DbError`).
    ///
    /// The transaction reads the database as it was when it started (a snapshot),
    /// with its own writes, and does not wait for the other transactions nor block
    /// them. A transaction that only reads cannot be committed or rolled back. A
    /// transaction that wrote is committed after the closure returns `Ok` and rolled
    /// back if the closure returns `Err` or if any of its mutable queries failed.
    /// The closure runs once.
    ///
    /// If the database changed since the transaction started, its writes are applied
    /// again to the current state when it commits:
    ///
    /// - The last transaction to commit a value of a key or an alias wins.
    /// - Amends of values (e.g. [`Amend::Add`](crate::Amend::Add)) apply to the
    ///   current value, so concurrent amends all count.
    /// - Removing an element, value or alias that is already gone does nothing.
    ///
    /// The commit fails with [`DbErrorType::Conflict`] (and has no effect) if an
    /// element it writes or connects with an edge was removed meanwhile, if it
    /// creates an index that was created meanwhile or if the storage was optimized
    /// or restored meanwhile. Use [`transaction_mut`](Self::transaction_mut) to
    /// write exclusively instead, which never conflicts.
    ///
    /// The writes are held in memory until the commit: unlike those of
    /// [`transaction_mut`](Self::transaction_mut) they are not spilled to the
    /// storage nor bounded by its memory limit, so very large ones (e.g. imports)
    /// are better written exclusively.
    ///
    /// Through its generic parameters it also allows transforming the query results
    /// into a type `T`.
    pub fn transaction<T, E: From<DbError>>(
        &self,
        f: impl FnOnce(&Transaction<Store>) -> Result<T, E>,
    ) -> Result<T, E> {
        self.root.transaction(f)
    }

    /// Executes mutable transaction. The transaction is running a closure `f`
    /// that will receive `&mut Transaction` to execute `exec` and `exec_mut` queries
    /// as if run on the main database object. You shall specify the return type `T`
    /// (can be `()`) and the error type `E` that must be constructible from the `DbError`
    /// (`E` can be `DbError`).
    ///
    /// Write transactions are committed if the closure returns `Ok` and rolled back if
    /// the closure returns `Err` or if any of its mutable queries failed. If the code
    /// panics and the program exits the write ahead log (WAL) makes sure the data in the
    /// main database file is restored to a consistent state prior to the transaction.
    ///
    /// Typical use case for a write transaction is to insert nodes and edges together.
    /// When not using a transaction you could end up only with nodes being inserted.
    ///
    /// Through its generic parameters the transaction also allows transforming the query
    /// results into a type `T`.
    pub fn transaction_mut<T, E: From<DbError>>(
        &self,
        f: impl FnOnce(&mut TransactionMut<Store>) -> Result<T, E>,
    ) -> Result<T, E> {
        self.root.transaction_mut(f)
    }

    /// Ensures all buffered data reaches durable storage. Call before
    /// reading raw database files for snapshots or backups. This is
    /// independent of `sync_mode` and always issues the sync syscall.
    pub fn sync(&self) -> Result<(), DbError> {
        self.root.with_storage_mut(|s| s.sync())
    }

    /// Returns the [`StorageConfig`] of this database.
    pub fn storage_config(&self) -> StorageConfig {
        self.root.info().config
    }

    /// Sets the [`StorageConfig`] of this database, see [`SyncMode`] and
    /// [`StorageData::set_overlay_limit()`].
    pub fn set_storage_config(&self, config: StorageConfig) {
        self.root.with_storage_mut(|s| {
            s.set_sync_mode(config.sync_mode);
            s.set_overlay_limit(config.overlay_limit);
            s.set_memory_limit(config.memory_limit);
        });
    }

    /// Returns the current [`SyncMode`] for this database.
    pub fn sync_mode(&self) -> SyncMode {
        self.root.info().config.sync_mode
    }

    /// Sets the sync mode for this database. See [`SyncMode`] for details.
    pub fn set_sync_mode(&self, mode: SyncMode) {
        self.root.with_storage_mut(|s| s.set_sync_mode(mode));
    }

    /// Returns the database size in bytes. Depending on the underlying storage
    /// the physical size in hardware might be higher. For example for in-memory
    /// storage this function reports used storage but actually allocated memory
    /// will likely be higher (expecting database growth in order to prevent too
    /// frequent allocations). For file based storages this number will be accurate
    /// but the actually used space on disk will be higher up to the next file system
    /// block size.
    ///
    /// It does not wait for the running write transaction and reports the
    /// size as of the last one that finished.
    pub fn size(&self) -> u64 {
        self.root.info().len
    }

    /// Atomically replaces the database content with the data at `filename`.
    ///
    /// Opens the file (or creates a new empty database if it does not exist),
    /// then swaps the internal storage under full MVCC synchronization.
    /// In-flight read transactions that started before the call see the old
    /// data; transactions that start after see the new data.
    ///
    /// Returns an error if the new database has a different page layout or
    /// is in a legacy format that requires migration.
    pub fn restore(&self, filename: &str) -> Result<(), DbError> {
        let data = Store::new(filename).map_err(|e| create_error(filename, e))?;
        let root = Root::open(data, Layout::default())?;
        self.root.restore(root)
    }
}

fn create_error(filename: &str, error: DbError) -> DbError {
    DbError::db(
        DbErrorType::DbCreate,
        format!("Failed to create database: {filename}"),
    )
    .caused_by(error)
}

impl DbAny {
    /// Creates a new DbAny instance using DbFile.
    pub fn new_file(filename: &str) -> Result<Self, DbError> {
        FileStorage::new(filename)
            .and_then(|storage| Self::with_data(AnyStorage::File(storage)))
            .map_err(|error| create_error(filename, error))
    }

    /// Creates a new DbAny instance using Db.
    pub fn new_mapped(filename: &str) -> Result<Self, DbError> {
        FileStorageMemoryMapped::new(filename)
            .and_then(|storage| Self::with_data(AnyStorage::MemoryMapped(storage)))
            .map_err(|error| create_error(filename, error))
    }

    /// Creates a new DbAny instance using DbMemory.
    pub fn new_memory(filename: &str) -> Result<Self, DbError> {
        MemoryStorage::new(filename)
            .and_then(|storage| Self::with_data(AnyStorage::Memory(storage)))
            .map_err(|error| create_error(filename, error))
    }
}

#[cfg(test)]
mod tests {
    mod test_model;

    use super::*;
    use crate::QueryBuilder;
    use crate::test_utilities::test_file::TestFile;
    use std::borrow::Cow;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

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
            if self.fail_writes.load(Ordering::Relaxed) {
                return Err(DbError::storage(
                    DbErrorType::NotAllowed,
                    "injected write failure",
                ));
            }
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
        let db = DbImpl::with_data(data).unwrap();
        (
            db,
            FailFlags {
                writes: fail_writes,
                reads: fail_reads,
            },
        )
    }

    /// Fails the commit of a write transaction and its restore.
    fn poison(db: &DbImpl<FailableFileStorage>, flags: &FailFlags) {
        let fail = flags.writes.clone();
        let _ = db.transaction_mut(|t| -> Result<(), DbError> {
            t.exec_mut(
                QueryBuilder::insert()
                    .values(vec![vec![("k", "modified").into()]])
                    .ids("n")
                    .query(),
            )?;
            fail.store(true, Ordering::Relaxed);
            Ok(())
        });
        flags.writes.store(false, Ordering::Relaxed);
    }

    #[test]
    fn transaction_should_write_without_a_snapshot() {
        let test_file = TestFile::new();
        let (db, _flags) = failable_db(test_file.file_name());

        let id = db
            .transaction(|t| -> Result<crate::DbId, DbError> {
                let id = t
                    .exec_mut(QueryBuilder::insert().nodes().aliases("n").query())?
                    .ids()[0];
                t.exec_mut(
                    QueryBuilder::insert()
                        .values([[("k", 1).into()]])
                        .ids("n")
                        .query(),
                )?;
                let result = t.exec(QueryBuilder::select().ids("n").query())?;
                assert_eq!(result.elements[0].values, vec![("k", 1).into()]);
                Ok(id)
            })
            .unwrap();

        let result = db.exec(QueryBuilder::select().ids("n").query()).unwrap();
        assert_eq!(result.elements[0].id, id);
        assert_eq!(result.elements[0].values, vec![("k", 1).into()]);
    }

    #[test]
    fn transaction_without_a_snapshot_should_drop_failed_writes() {
        let test_file = TestFile::new();
        let (db, _flags) = failable_db(test_file.file_name());

        let error = db
            .transaction(|t| -> Result<(), DbError> {
                t.exec_mut(QueryBuilder::insert().nodes().aliases("n").query())?;
                t.exec_mut(
                    QueryBuilder::insert()
                        .edges()
                        .from("n")
                        .to("missing")
                        .query(),
                )?;
                Ok(())
            })
            .unwrap_err();

        assert_eq!(error.description, "Alias 'missing' not found");
        assert!(db.exec(QueryBuilder::select().ids("n").query()).is_err());
    }

    #[test]
    fn rollback_failure_with_successful_recovery() {
        let test_file = TestFile::new();
        let (db, flags) = failable_db(test_file.file_name());

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

        flags.writes.store(false, Ordering::Relaxed);
        let r = db.exec(QueryBuilder::select().ids("n").query()).unwrap();
        assert_eq!(r.elements[0].values, vec![("k", "original").into()]);
    }

    #[test]
    fn rollback_with_failing_storage_should_return_the_closure_error() {
        let test_file = TestFile::new();
        let (db, flags) = failable_db(test_file.file_name());

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

        assert_eq!(err.description, "user error");
        flags.writes.store(false, Ordering::Relaxed);
        flags.reads.store(false, Ordering::Relaxed);
        let r = db.exec(QueryBuilder::select().ids("n").query()).unwrap();
        assert_eq!(r.elements[0].values, vec![("k", "original").into()]);
    }

    #[test]
    fn poisoned_db_should_refuse_reads_until_a_write_restores_it() {
        let test_file = TestFile::new();
        let (db, flags) = failable_db(test_file.file_name());

        db.exec_mut(
            QueryBuilder::insert()
                .nodes()
                .aliases("n")
                .values(vec![vec![("k", "original").into()]])
                .query(),
        )
        .unwrap();

        poison(&db, &flags);

        let err = db
            .exec(QueryBuilder::select().ids("n").query())
            .unwrap_err();
        assert_eq!(err.ty, DbErrorType::Poisoned);

        db.exec_mut(
            QueryBuilder::insert()
                .values(vec![vec![("k", "v").into()]])
                .ids("n")
                .query(),
        )
        .unwrap();
        let r = db.exec(QueryBuilder::select().ids("n").query()).unwrap();
        assert_eq!(r.elements[0].values, vec![("k", "v").into()]);
    }

    #[test]
    fn reopening_after_recovered_transaction_restores_data() {
        let test_file = TestFile::new();
        let db_path = test_file.file_name().clone();

        {
            let (db, flags) = failable_db(&db_path);
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
            let (db, flags) = failable_db(&db_path);
            db.exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .aliases("n")
                    .values(vec![vec![("k", "original").into()]])
                    .query(),
            )
            .unwrap();

            poison(&db, &flags);
        }

        let (db, _) = failable_db(&db_path);
        let r = db.exec(QueryBuilder::select().ids("n").query()).unwrap();
        assert_eq!(r.elements[0].values, vec![("k", "original").into()]);
    }

    mod db_impl {
        use super::test_model::Dump;
        use super::test_model::Run;
        use super::*;
        use crate::DbId;
        use crate::DbValue;
        use crate::storage::DEFAULT_OVERLAY_LIMIT;
        use crate::storage::any_storage::AnyStorageImpl;
        use crate::storage::file_storage::FileStorageImpl;
        use crate::storage::file_storage_memory_mapped::FileStorageMemoryMappedImpl;
        use crate::storage::memory_storage::MemoryStorageImpl;
        use agplatform::Fs;
        use agplatform::Platform;
        use agplatform::test_platform::TestPlatform;
        use agplatform::test_platform::test_fs::FsOp;
        use agplatform::test_platform::test_platform;
        use std::io::ErrorKind;

        const NAME: &str = "/db";
        const WAL: &str = "/.db";
        const REBUILD: &str = "/db.rebuild";
        const MODEL_STEPS: u64 = 300;
        const FAULT_STEPS: u64 = 100;
        const CRASH_STEPS: u64 = 60;

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
                memory_limit: None,
            }
        }

        /// The nodes 1..=3 with the label "label" and ("k", 1), ("k", 2)
        /// and ("label", 3), the "label" indexed, on each storage kind.
        fn labelled() -> Vec<DbImpl<AnyStorageImpl<TestPlatform>>> {
            let platform = || Arc::new(test_platform());

            [
                AnyStorageImpl::Memory(memory(&platform())),
                AnyStorageImpl::File(file(&platform())),
                AnyStorageImpl::MemoryMapped(mapped(&platform())),
            ]
            .into_iter()
            .map(|storage| {
                let db = open(storage, StorageConfig::default());
                db.exec_mut(QueryBuilder::insert().index("label").query())
                    .unwrap();
                db.exec_mut(
                    QueryBuilder::insert()
                        .nodes()
                        .values([
                            vec!["label".into(), ("k", 1).into()],
                            vec![("k", 2).into()],
                            vec![("label", 3).into()],
                        ])
                        .query(),
                )
                .unwrap();
                db
            })
            .collect()
        }

        fn open<D: StorageData>(data: D, config: StorageConfig) -> DbImpl<D> {
            let db = DbImpl::with_data(data).unwrap();
            db.set_storage_config(config);
            db
        }

        fn inserted<D: StorageData>(data: D, config: StorageConfig) -> DbImpl<D> {
            let db = open(data, config);
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

        fn modify<D: StorageData>(db: &DbImpl<D>) -> Result<QueryResult, DbError> {
            db.exec_mut(
                QueryBuilder::insert()
                    .values(vec![vec![("k", "modified").into()]])
                    .ids("n")
                    .query(),
            )
        }

        fn modify_in<D: StorageData>(t: &mut TransactionMut<D>) -> Result<QueryResult, DbError> {
            t.exec_mut(
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

        fn poisoned<D: StorageData>(db: &DbImpl<D>) -> bool {
            db.exec(QueryBuilder::select().ids("n").query())
                .is_err_and(|error| error.ty == DbErrorType::Poisoned)
        }

        /// Fails each read of the database file the `action` does, one at
        /// a time, on the subject `open` created, and checks that the
        /// failure is returned until the `action` runs out of reads and
        /// returns what it returns without failures.
        fn assert_read_failures_on<T, R: std::fmt::Debug + PartialEq>(
            open: impl Fn(&Arc<TestPlatform>) -> T,
            action: impl Fn(&mut T) -> Result<R, DbError>,
        ) {
            let expected = action(&mut open(&Arc::new(test_platform()))).unwrap();
            let mut failures = 0;

            for nth in 0.. {
                let platform = Arc::new(test_platform());
                let mut subject = open(&platform);
                let mut results = vec![None; nth];
                results.push(Some(ErrorKind::Other));
                platform.fs().set_io_errors(FsOp::Read, NAME, results);

                let result = action(&mut subject);

                platform.fs().clear_faults();

                if let Ok(result) = result {
                    assert_eq!(result, expected, "read {nth} failed silently");
                    break;
                }

                failures += 1;
            }

            assert!(failures > 0, "the action should read");
        }

        fn memory(platform: &Arc<TestPlatform>) -> MemoryStorageImpl<TestPlatform> {
            MemoryStorageImpl::with_platform(platform.clone(), NAME).unwrap()
        }

        /// The storage kinds with the overlay limit of each: the default
        /// and 0, spilling every page ahead of the commit.
        fn model_configs() -> [StorageConfig; 2] {
            [
                config(SyncMode::None, DEFAULT_OVERLAY_LIMIT),
                config(SyncMode::None, 0),
            ]
        }

        /// Runs the random queries of `seed` on the `db`, checking it
        /// against the model after each and calling `between` every
        /// `every` steps.
        fn model_run<D: StorageData>(
            mut db: DbImpl<D>,
            seed: u64,
            every: u64,
            mut between: impl FnMut(DbImpl<D>, &Run) -> DbImpl<D>,
        ) {
            let mut run = Run::new(seed);

            for step in 1..=MODEL_STEPS {
                run.step(&db);

                if step % every == 0 {
                    db = between(db, &run);
                }
            }
        }

        fn optimized<D: StorageData>(db: DbImpl<D>, run: &Run) -> DbImpl<D> {
            db.optimize_storage().unwrap();
            run.check(&db, "optimize_storage");
            db
        }

        /// Fails the `fs_op` once at each of its calls, one at a time,
        /// in each random query of `seed` on a file database and checks
        /// after each failure that the database still holds the model.
        fn model_faults(fs_op: FsOp, config: StorageConfig, seed: u64) {
            let platform = Arc::new(test_platform());
            let db = open(file(&platform), config);
            let mut run = Run::new(seed);

            for _ in 0..FAULT_STEPS {
                let op = run.op();

                for nth in 0.. {
                    let mut results = vec![None; nth];
                    results.extend([Some(ErrorKind::Other), None]);
                    platform.fs().set_io_errors(fs_op, "", results);

                    let result = op.run(&db);

                    platform.fs().clear_faults();

                    if result.is_ok() || run.model.fails(&op) {
                        run.apply(&op, &result);
                        run.check(&db, &format!("{op:?}"));
                        break;
                    }

                    let error = result.unwrap_err().to_string();
                    assert!(error.contains("simulated"), "{fs_op:?} {nth}: {error}");
                    run.check(&db, &format!("{fs_op:?} {nth} failed: {op:?}"));
                }
            }
        }

        fn saved(platform: &Arc<TestPlatform>) -> Vec<(&'static str, Vec<u8>)> {
            [NAME, WAL]
                .into_iter()
                .filter(|path| platform.fs().exists(path).unwrap())
                .map(|path| (path, platform.fs().read(path).unwrap()))
                .collect()
        }

        fn restored(files: &[(&'static str, Vec<u8>)]) -> Arc<TestPlatform> {
            let platform = Arc::new(test_platform());

            for (path, bytes) in files {
                platform.fs().write(path, bytes).unwrap();
            }

            platform.fs().sync();
            platform
        }

        /// Crashes a file database at each write of each random query of
        /// `seed`, one at a time (all writes from it on fail), and checks
        /// that reopened it holds the model before or after the query, and
        /// after it if the query returned unless a power loss may roll back
        /// the last commit (`SyncMode::Commit`). `power_loss` loses the
        /// writes not synced, otherwise only the process dies.
        fn model_crashes(config: StorageConfig, power_loss: bool, seed: u64) {
            let mut run = Run::new(seed);
            let mut files = Vec::new();
            let durable = !power_loss || config.sync_mode == SyncMode::Strict;

            for _ in 0..CRASH_STEPS {
                let op = run.op();
                let before = run.model.clone();
                let platform = restored(&files);
                let db = open(file(&platform), config);
                let result = op.run(&db);
                run.apply(&op, &result);
                run.check(&db, &format!("{op:?}"));
                drop(db);
                let pre = std::mem::replace(&mut files, saved(&platform));

                if before.fails(&op) {
                    continue;
                }

                for nth in 0.. {
                    let platform = restored(&pre);
                    let db = open(file(&platform), config);
                    let mut results = vec![None; nth];
                    results.push(Some(ErrorKind::Other));
                    platform.fs().set_io_errors(FsOp::Write, "", results);

                    let result = op.run(&db);

                    if !power_loss {
                        platform.fs().sync();
                    }

                    platform.fs().crash();
                    platform.fs().clear_faults();
                    std::mem::forget(db);
                    let db = open(file(&platform), config);
                    let after = Dump::of(&db, &run.model) == run.model.dump();
                    let kept = after || Dump::of(&db, &before) == before.dump();
                    let context = format!("{config:?} crash at write {nth}: {op:?}");

                    if result.is_ok() {
                        assert!(
                            after || !durable && kept,
                            "{context} returned, did not last"
                        );
                        break;
                    }

                    assert!(kept, "{context}");
                }
            }
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
            let db = inserted(storage(&platform), config);
            fail_once(&platform, op, path);

            let failed = modify(&db).is_err();

            let poisoned = poisoned(&db);
            let current = value(&db);
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

            fn assert_readers_during_failed_commits<D: StorageData + Send + Sync>(
                db: &DbImpl<D>,
                mut commit: impl FnMut() -> Result<QueryResult, DbError>,
            ) {
                const COMMITS: usize = 300;
                let running = std::sync::atomic::AtomicBool::new(true);

                std::thread::scope(|scope| {
                    let readers: Vec<_> = (0..2)
                        .map(|_| {
                            scope.spawn(|| {
                                let mut reads = 0_u64;

                                while running.load(std::sync::atomic::Ordering::Relaxed) {
                                    assert_eq!(value(db), "original".into());
                                    reads += 1;
                                }

                                reads
                            })
                        })
                        .collect();

                    for _ in 0..COMMITS {
                        assert!(commit().is_err());
                    }

                    running.store(false, std::sync::atomic::Ordering::Relaxed);

                    for reader in readers {
                        assert_ne!(reader.join().unwrap(), 0);
                    }
                });
            }

            #[test]
            fn transaction_mut_failed_flush_should_not_be_seen_by_readers() {
                let platform = Arc::new(test_platform());
                let db = inserted(mapped(&platform), StorageConfig::default());

                assert_readers_during_failed_commits(&db, || {
                    fail_once(&platform, FsOp::Write, WAL);
                    modify(&db)
                });

                assert!(!poisoned(&db));
            }

            #[test]
            fn transaction_mut_commit_past_memory_limit_should_not_fail_readers() {
                let platform = Arc::new(test_platform());
                let db = inserted(memory(&platform), StorageConfig::default());
                db.set_storage_config(StorageConfig {
                    memory_limit: Some(db.size() + 4096),
                    ..db.storage_config()
                });

                assert_readers_during_failed_commits(&db, || {
                    db.exec_mut(
                        QueryBuilder::insert()
                            .values(vec![vec![("k", vec![1_u8; 64 * 1024]).into()]])
                            .ids("n")
                            .query(),
                    )
                });
            }

            #[test]
            fn transaction_mut_failed_query_should_refuse_the_transaction() {
                let platform = Arc::new(test_platform());
                let db = inserted(file(&platform), StorageConfig::default());

                let err = db
                    .transaction_mut(|t| -> Result<(), DbError> {
                        modify_in(t)?;
                        let failed = t
                            .exec_mut(QueryBuilder::insert().aliases("a").ids(100).query())
                            .unwrap_err();
                        assert_eq!(
                            t.exec(QueryBuilder::select().ids("n").query()),
                            Err(failed.clone())
                        );
                        assert_eq!(modify_in(t), Err(failed));
                        Ok(())
                    })
                    .unwrap_err();

                assert_eq!(err.description, "Id '100' not found");
                assert_eq!(value(&db), "original".into());
            }

            #[test]
            fn transaction_mut_should_commit_after_failed_commit() {
                let platform = Arc::new(test_platform());
                let db = inserted(file(&platform), StorageConfig::default());
                fail_once(&platform, FsOp::Write, NAME);
                assert!(modify(&db).is_err());

                modify(&db).unwrap();

                assert_eq!(value(&db), "modified".into());
                drop(db);
                assert_eq!(
                    value(&open(file(&platform), StorageConfig::default())),
                    "modified".into()
                );
            }

            #[test]
            fn transaction_mut_should_return_query_error_after_failed_write() {
                let platform = Arc::new(test_platform());
                let db = inserted(file(&platform), config(SyncMode::None, 0));
                fail_once(&platform, FsOp::Write, NAME);

                let err = modify(&db).unwrap_err();

                assert_eq!(err.description, "[Fs] simulated Write failure: /db");
                assert!(!poisoned(&db));
                assert_eq!(value(&db), "original".into());
                modify(&db).unwrap();
                assert_eq!(value(&db), "modified".into());
            }

            #[test]
            fn transaction_mut_failed_recovery_after_failed_commit_should_poison() {
                let platform = Arc::new(test_platform());
                let db = inserted(file(&platform), StorageConfig::default());
                platform
                    .fs()
                    .set_io_error(FsOp::Write, NAME, ErrorKind::Other);

                let err = modify(&db).unwrap_err();

                assert_eq!(err.ty, DbErrorType::Poisoned);
                assert!(poisoned(&db));
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
                let db = inserted(file(&platform), StorageConfig::default());
                let len = db.size();
                fail_once(&platform, FsOp::Write, REBUILD);

                assert!(db.optimize_storage().is_err());

                assert!(!poisoned(&db));
                assert_eq!(db.size(), len);
                assert_eq!(value(&db), "original".into());
                db.optimize_storage().unwrap();
                assert!(db.size() <= len);
                assert_eq!(value(&db), "original".into());
                assert!(!platform.fs().exists(REBUILD).unwrap());
            }

            #[test]
            fn optimize_storage_with_longer_reader_should_fail_and_keep_storage() {
                let platform = Arc::new(test_platform());
                let db = inserted(file(&platform), StorageConfig::default());
                let len = db.size();
                let reading = std::sync::Barrier::new(2);
                let optimized = std::sync::atomic::AtomicBool::new(false);

                let result = std::thread::scope(|scope| {
                    scope.spawn(|| {
                        db.transaction(|_| -> Result<(), DbError> {
                            reading.wait();

                            while !optimized.load(std::sync::atomic::Ordering::Relaxed) {
                                std::thread::yield_now();
                            }

                            Ok(())
                        })
                    });
                    reading.wait();
                    let result = db.optimize_storage();
                    optimized.store(true, std::sync::atomic::Ordering::Relaxed);
                    result
                });

                assert!(result.is_err());
                assert_eq!(db.size(), len);
                assert_eq!(value(&db), "original".into());
                db.optimize_storage().unwrap();
                assert_eq!(value(&db), "original".into());
            }

            #[test]
            fn optimize_storage_should_keep_the_model_state() {
                for seed in 1..=2 {
                    for config in model_configs() {
                        let platform = Arc::new(test_platform());
                        model_run(open(memory(&platform), config), seed, 25, optimized);
                        let platform = Arc::new(test_platform());
                        model_run(open(file(&platform), config), seed, 25, optimized);
                        let platform = Arc::new(test_platform());
                        model_run(open(mapped(&platform), config), seed, 25, optimized);
                    }
                }
            }
        }

        /// The nodes a, b, c, d (1..=4) with the values 4, 3, 2, 1 of "k",
        /// the `LONG` key and value of a stored outside of its key-value
        /// and the edges a -> b (-1), a -> c (-2), b -> d (-3), c -> d (-4).
        const LONG: &str = "longer than the 15 inline bytes";

        fn searched(platform: &Arc<TestPlatform>) -> DbImpl<FileStorageImpl<TestPlatform>> {
            let db = open(file(platform), StorageConfig::default());
            db.exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .aliases(["a", "b", "c", "d"])
                    .values(vec![
                        vec![("k", 4).into(), (LONG, LONG).into()],
                        vec![("k", 3).into()],
                        vec![("k", 2).into()],
                        vec![("k", 1).into()],
                    ])
                    .query(),
            )
            .unwrap();
            db.exec_mut(
                QueryBuilder::insert()
                    .edges()
                    .from(["a", "a", "b", "c"])
                    .to(["b", "c", "d", "d"])
                    .query(),
            )
            .unwrap();
            db.exec_mut(QueryBuilder::insert().index("k").query())
                .unwrap();
            db
        }

        mod exec {
            use super::*;
            use crate::Comparison;
            use crate::DbKeyOrder;

            #[test]
            fn exec_read_failure_should_return_error() {
                for query in [
                    QueryBuilder::search().from("a").query(),
                    QueryBuilder::search().depth_first().to("d").query(),
                    QueryBuilder::search().from("a").to("d").query(),
                    QueryBuilder::search().to("d").from("a").query(),
                    QueryBuilder::search().elements().query(),
                    QueryBuilder::search()
                        .from("a")
                        .order_by(DbKeyOrder::Asc("k".into()))
                        .query(),
                    QueryBuilder::search()
                        .from("a")
                        .where_()
                        .edge_count_from_greater_than(1)
                        .or()
                        .edge_count_to(1)
                        .or()
                        .edge_count(2)
                        .query(),
                    QueryBuilder::search().from("a").where_().ids("c").query(),
                    QueryBuilder::search()
                        .from("a")
                        .to("d")
                        .where_()
                        .edge_count_from(1)
                        .query(),
                    QueryBuilder::search()
                        .to("d")
                        .from("a")
                        .where_()
                        .edge_count_from(1)
                        .query(),
                    QueryBuilder::search()
                        .from("a")
                        .limit(2)
                        .where_()
                        .edge_count_from(1)
                        .query(),
                    QueryBuilder::search()
                        .to("d")
                        .offset(1)
                        .where_()
                        .edge_count_from(1)
                        .query(),
                    QueryBuilder::search()
                        .elements()
                        .offset(1)
                        .limit(1)
                        .where_()
                        .edge_count_from(1)
                        .query(),
                    QueryBuilder::search()
                        .elements()
                        .order_by(DbKeyOrder::Asc("k".into()))
                        .query(),
                    QueryBuilder::search()
                        .to("d")
                        .order_by(DbKeyOrder::Asc("k".into()))
                        .query(),
                    QueryBuilder::search()
                        .index("k")
                        .value(3)
                        .order_by(DbKeyOrder::Asc("k".into()))
                        .where_()
                        .edge_count_from(1)
                        .query(),
                    QueryBuilder::search()
                        .depth_first()
                        .from("a")
                        .where_()
                        .not_beyond()
                        .edge()
                        .query(),
                    QueryBuilder::search()
                        .depth_first()
                        .to("d")
                        .where_()
                        .not_beyond()
                        .edge()
                        .query(),
                    QueryBuilder::search()
                        .to("d")
                        .where_()
                        .not_beyond()
                        .edge()
                        .query(),
                    QueryBuilder::search()
                        .from("a")
                        .where_()
                        .key("k")
                        .value(Comparison::Equal(3.into()))
                        .query(),
                    QueryBuilder::search().from("a").where_().keys("k").query(),
                    QueryBuilder::search()
                        .elements()
                        .where_()
                        .key(LONG)
                        .value(Comparison::Equal(LONG.into()))
                        .query(),
                    QueryBuilder::search()
                        .elements()
                        .order_by(DbKeyOrder::Desc(LONG.into()))
                        .offset(1)
                        .limit(2)
                        .query(),
                ] {
                    assert_read_failures_on(searched, |db| db.exec(&query));
                }

                assert_read_failures_on(searched, |db| {
                    db.exec(QueryBuilder::select().edge_count().ids(["a", "d"]).query())
                });
                assert_read_failures_on(searched, |db| {
                    db.exec(QueryBuilder::select().ids(["a", "d"]).query())
                });
                assert_read_failures_on(searched, |db| {
                    db.exec(QueryBuilder::select().values("k").ids(["a", "d"]).query())
                });
                assert_read_failures_on(searched, |db| {
                    db.exec(QueryBuilder::select().keys().ids(["a", "d"]).query())
                });
                assert_read_failures_on(searched, |db| {
                    db.exec(QueryBuilder::select().values(LONG).ids("a").query())
                });
                assert_read_failures_on(searched, |db| {
                    db.exec(QueryBuilder::select().aliases().search().from("a").query())
                });
            }

            #[test]
            fn exec_search_by_label_should_match_only_labels() {
                for db in labelled() {
                    let search = |comparison: Comparison| {
                        db.exec(
                            QueryBuilder::search()
                                .elements()
                                .where_()
                                .key("label")
                                .value(comparison)
                                .query(),
                        )
                        .unwrap()
                        .ids()
                    };

                    // THEN
                    assert_eq!(search(Comparison::Equal(DbValue::None)), vec![DbId(1)]);
                    assert_eq!(search(Comparison::NotEqual(DbValue::None)), vec![DbId(3)]);
                    assert_eq!(search(Comparison::LessThan(DbValue::None)), vec![]);
                    assert_eq!(search(Comparison::GreaterThan(0.into())), vec![DbId(3)]);

                    // AND
                    assert_eq!(
                        db.exec(
                            QueryBuilder::search()
                                .index("label")
                                .value(DbValue::None)
                                .query()
                        )
                        .unwrap()
                        .ids(),
                        vec![DbId(1)]
                    );
                    assert_eq!(
                        db.exec(
                            QueryBuilder::search()
                                .elements()
                                .where_()
                                .keys("label")
                                .query()
                        )
                        .unwrap()
                        .ids(),
                        vec![DbId(1), DbId(3)]
                    );
                }
            }
        }

        mod exec_mut {
            use super::*;
            use crate::DbElement;

            #[test]
            fn exec_mut_insert_labels_should_select_them_as_none() {
                for db in labelled() {
                    // WHEN
                    let selected = db
                        .exec(QueryBuilder::select().values("label").ids([1, 3]).query())
                        .unwrap();

                    // THEN
                    assert_eq!(
                        selected.elements,
                        vec![
                            DbElement {
                                id: DbId(1),
                                from: DbId(0),
                                to: DbId(0),
                                values: vec!["label".into()],
                            },
                            DbElement {
                                id: DbId(3),
                                from: DbId(0),
                                to: DbId(0),
                                values: vec![("label", 3).into()],
                            },
                        ]
                    );

                    // AND
                    assert_eq!(
                        db.exec(QueryBuilder::select().ids(1).query())
                            .unwrap()
                            .elements[0]
                            .values,
                        vec!["label".into(), ("k", 1).into()]
                    );
                    assert_eq!(
                        db.exec(QueryBuilder::select().keys().ids(1).query())
                            .unwrap()
                            .elements[0]
                            .values,
                        vec!["label".into(), "k".into()]
                    );
                    assert!(
                        db.exec(QueryBuilder::select().values("label").ids(2).query())
                            .is_err()
                    );
                }
            }

            #[test]
            fn exec_mut_read_failure_should_return_error() {
                assert_read_failures_on(searched, |db| {
                    db.exec_mut(QueryBuilder::remove().ids("a").query())
                });
                assert_read_failures_on(searched, |db| {
                    db.exec_mut(QueryBuilder::remove().ids(-3).query())
                });
                assert_read_failures_on(searched, |db| {
                    db.exec_mut(QueryBuilder::insert().index("j").query())
                });
                assert_read_failures_on(searched, |db| {
                    db.exec_mut(
                        QueryBuilder::insert()
                            .values_uniform([("k", 5).into(), ("j", 6).into()])
                            .ids("a")
                            .query(),
                    )
                });
                assert_read_failures_on(searched, |db| {
                    db.exec_mut(QueryBuilder::remove().values("k").ids("a").query())
                });
                assert_read_failures_on(searched, |db| {
                    db.exec_mut(QueryBuilder::remove().values(LONG).ids("a").query())
                });
                assert_read_failures_on(searched, |db| {
                    db.exec_mut(QueryBuilder::remove().index("k").query())
                });
            }

            #[test]
            fn exec_mut_remove_index_read_failure_should_keep_the_index() {
                for nth in 0.. {
                    // GIVEN
                    let platform = Arc::new(test_platform());
                    let db = searched(&platform);
                    let mut results = vec![None; nth];
                    results.extend([Some(ErrorKind::Other), None]);
                    platform.fs().set_io_errors(FsOp::Read, NAME, results);

                    // WHEN
                    let result = db.exec_mut(QueryBuilder::remove().index("k").query());

                    // THEN
                    let found = db
                        .exec(QueryBuilder::search().index("k").value(3).query())
                        .map(|found| found.ids());

                    if result.is_ok() {
                        assert!(nth > 0);
                        assert!(found.is_err());
                        break;
                    }

                    assert_eq!(found, Ok(vec![DbId(2)]), "read {nth}");
                }
            }

            #[test]
            fn exec_mut_should_match_the_model_over_random_queries() {
                for seed in 1..=4 {
                    for config in model_configs() {
                        let platform = Arc::new(test_platform());
                        model_run(open(memory(&platform), config), seed, 1, |db, _| db);
                        let platform = Arc::new(test_platform());
                        model_run(open(file(&platform), config), seed, 1, |db, _| db);
                        let platform = Arc::new(test_platform());
                        model_run(open(mapped(&platform), config), seed, 1, |db, _| db);
                    }
                }
            }

            #[test]
            fn exec_mut_read_failure_should_keep_the_model_state() {
                for seed in 1..=2 {
                    for config in model_configs() {
                        model_faults(FsOp::Read, config, seed);
                    }
                }
            }

            #[test]
            fn exec_mut_write_failure_should_keep_the_model_state() {
                for seed in 1..=2 {
                    for config in model_configs() {
                        model_faults(FsOp::Write, config, seed);
                    }

                    model_faults(
                        FsOp::SyncData,
                        config(SyncMode::Commit, DEFAULT_OVERLAY_LIMIT),
                        seed,
                    );
                }
            }

            #[test]
            fn exec_mut_crash_should_recover_the_state_before_or_after_the_query() {
                for seed in 1..=2 {
                    for config in model_configs() {
                        model_crashes(config, false, seed);
                    }

                    for sync_mode in [SyncMode::Commit, SyncMode::Strict] {
                        for overlay_limit in [DEFAULT_OVERLAY_LIMIT, 0] {
                            model_crashes(config(sync_mode, overlay_limit), true, seed);
                        }
                    }
                }
            }
        }

        mod with_data {
            use super::*;

            #[test]
            fn with_data_read_failure_should_return_error() {
                assert_read_failures_on(
                    |platform| {
                        drop(searched(platform));
                        platform.clone()
                    },
                    |platform| {
                        let data = FileStorageImpl::with_platform(platform.clone(), NAME)?;
                        let db = DbImpl::with_data(data)?;
                        db.exec(QueryBuilder::select().indexes().query())
                    },
                );
            }

            #[test]
            fn with_data_should_reopen_the_model_state() {
                for seed in 1..=2 {
                    for config in model_configs() {
                        let platform = Arc::new(test_platform());
                        model_run(open(file(&platform), config), seed, 20, |db, run| {
                            drop(db);
                            let db = open(file(&platform), config);
                            run.check(&db, "reopened");
                            db
                        });
                        let platform = Arc::new(test_platform());
                        model_run(open(mapped(&platform), config), seed, 20, |db, run| {
                            drop(db);
                            let db = open(mapped(&platform), config);
                            run.check(&db, "reopened");
                            db
                        });
                    }
                }
            }
        }
    }
}
