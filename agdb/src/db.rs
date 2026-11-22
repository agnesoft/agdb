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
use crate::storage::file_storage::FileStorage;
use crate::storage::file_storage_memory_mapped::FileStorageMemoryMapped;
use crate::storage::memory_storage::MemoryStorage;
use crate::storage::pages::Reader;
use crate::storage::root::Layout;
use crate::storage::root::Root;
use crate::transaction::ReadView;

/// Version of the database format. A database of an older version is
/// migrated when opened and a newer one is refused.
pub const DB_VERSION: u64 = 2;

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
/// aborting any unfinished transactions.
///
/// On load, if the WAL file is present (e.g. due to a crash), it will be processed
/// restoring any consistent state that existed before the crash. Data is only
/// written to the main file if the reverse operation has been committed to the
/// WAL file. The WAL is then purged on commit of a transaction (all queries are
/// transactional even if the transaction is not explicitly used).
pub struct DbImpl<Store: StorageData> {
    root: Root<Store>,
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
    /// let mut db = DbMemory::with_data(MemoryStorage::new("test").unwrap()).unwrap();
    /// ```
    pub fn with_data(data: Store) -> Result<Self, DbError> {
        Ok(Self {
            root: Root::open(data, Layout::default())?,
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
    pub fn backup(&self, filename: &str) -> Result<(), DbError> {
        self.root.storage().backup(filename)
    }

    /// Copies the database to `filename` path.  Consider calling
    /// `optimize_storage()` prior to this function to reduce the
    /// size of the storage file to be copied.
    pub fn copy(&self, filename: &str) -> Result<Self, DbError> {
        let storage = self.root.storage();
        let mut copy = storage.copy(filename)?;
        copy.set_sync_mode(storage.sync_mode());
        copy.set_overlay_limit(storage.overlay_limit());
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
    pub fn exec_mut<T: QueryMut>(&mut self, query: T) -> Result<QueryResult, DbError> {
        self.transaction_mut(|transaction| transaction.exec_mut(query))
    }

    /// Returns the filename that was used to
    /// construct the database.
    pub fn filename(&self) -> &str {
        self.root.storage().name()
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
    pub fn optimize_storage(&mut self) -> Result<(), DbError> {
        Ok(())
    }

    /// Changes the name of the database changing also the names of the files
    /// (if the storage is file based).
    pub fn rename(&mut self, filename: &str) -> Result<(), DbError> {
        self.root.storage_mut().rename(filename)
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
        let transaction = match self.root.read() {
            Ok((reader, meta)) => Transaction::new(ReadView::Committed(reader), meta),
            Err(error) => Transaction::failed(Reader::new(self.root.storage()), error),
        };

        f(&transaction)
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
        &mut self,
        f: impl FnOnce(&mut TransactionMut<Store>) -> Result<T, E>,
    ) -> Result<T, E> {
        let mut transaction = TransactionMut::new(self.root.begin()?);
        let result = f(&mut transaction)?;
        transaction.commit()?;
        Ok(result)
    }

    /// Ensures all buffered data reaches durable storage. Call before
    /// reading raw database files for snapshots or backups. This is
    /// independent of `sync_mode` and always issues the sync syscall.
    pub fn sync(&mut self) -> Result<(), DbError> {
        self.root.storage_mut().sync()
    }

    /// Returns the [`StorageConfig`] of this database.
    pub fn storage_config(&self) -> StorageConfig {
        StorageConfig {
            sync_mode: self.root.storage().sync_mode(),
            overlay_limit: self.root.storage().overlay_limit(),
        }
    }

    /// Sets the [`StorageConfig`] of this database, see [`SyncMode`] and
    /// [`StorageData::set_overlay_limit()`].
    pub fn set_storage_config(&mut self, config: StorageConfig) {
        let storage = self.root.storage_mut();
        storage.set_sync_mode(config.sync_mode);
        storage.set_overlay_limit(config.overlay_limit);
    }

    /// Returns the current [`SyncMode`] for this database.
    pub fn sync_mode(&self) -> SyncMode {
        self.root.storage().sync_mode()
    }

    /// Sets the sync mode for this database. See [`SyncMode`] for details.
    pub fn set_sync_mode(&mut self, mode: SyncMode) {
        self.root.storage_mut().set_sync_mode(mode);
    }

    /// Returns the database size in bytes. Depending on the underlying storage
    /// the physical size in hardware might be higher. For example for in-memory
    /// storage this function reports used storage but actually allocated memory
    /// will likely be higher (expecting database growth in order to prevent too
    /// frequent allocations). For file based storages this number will be accurate
    /// but the actually used space on disk will be higher up to the next file system
    /// block size.
    pub fn size(&self) -> u64 {
        self.root.storage().len()
    }

    /// Shrinks all the allocated internal storage buffers to the current size of the data
    /// and optimizes the storage. This may further reduce the size of the database
    /// both on disk and in memory at the cost of requiring more allocations upon inserting
    /// more data. Best to use only when the database is used for read-only operations after
    /// initial data population.
    pub fn shrink_to_fit(&mut self) -> Result<(), DbError> {
        self.optimize_storage()
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
    fn poison(db: &mut DbImpl<FailableFileStorage>, flags: &FailFlags) {
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

        flags.writes.store(false, Ordering::Relaxed);
        let r = db.exec(QueryBuilder::select().ids("n").query()).unwrap();
        assert_eq!(r.elements[0].values, vec![("k", "original").into()]);
    }

    #[test]
    fn rollback_with_failing_storage_should_return_the_closure_error() {
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

        assert_eq!(err.description, "user error");
        flags.writes.store(false, Ordering::Relaxed);
        flags.reads.store(false, Ordering::Relaxed);
        let r = db.exec(QueryBuilder::select().ids("n").query()).unwrap();
        assert_eq!(r.elements[0].values, vec![("k", "original").into()]);
    }

    #[test]
    fn poisoned_db_should_refuse_reads_until_a_write_restores_it() {
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

        poison(&mut db, &flags);

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

            poison(&mut db, &flags);
        }

        let (db, _) = failable_db(&db_path);
        let r = db.exec(QueryBuilder::select().ids("n").query()).unwrap();
        assert_eq!(r.elements[0].values, vec![("k", "original").into()]);
    }

    mod db_impl {
        use super::*;
        use crate::DbId;
        use crate::DbValue;
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
            let mut db = DbImpl::with_data(data).unwrap();
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

            #[test]
            fn transaction_mut_failed_query_should_refuse_the_transaction() {
                let platform = Arc::new(test_platform());
                let mut db = inserted(file(&platform), StorageConfig::default());

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
            fn transaction_mut_should_return_query_error_after_failed_write() {
                let platform = Arc::new(test_platform());
                let mut db = inserted(file(&platform), config(SyncMode::None, 0));
                fail_once(&platform, FsOp::Write, NAME);

                let err = modify(&mut db).unwrap_err();

                assert_eq!(err.description, "[Fs] simulated Write failure: /db");
                assert!(!poisoned(&db));
                assert_eq!(value(&db), "original".into());
                modify(&mut db).unwrap();
                assert_eq!(value(&db), "modified".into());
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
                let mut db = inserted(file(&platform), StorageConfig::default());
                let len = db.size();
                fail_once(&platform, FsOp::Write, NAME);

                assert!(db.optimize_storage().is_err());

                assert!(!poisoned(&db));
                assert_eq!(db.size(), len);
                assert_eq!(value(&db), "original".into());
                db.optimize_storage().unwrap();
                assert!(db.size() < len);
            }
        }

        /// The nodes a, b, c, d (1..=4) with the values 4, 3, 2, 1 of "k",
        /// the `LONG` key and value of a stored outside of its key-value
        /// and the edges a -> b (-1), a -> c (-2), b -> d (-3), c -> d (-4).
        const LONG: &str = "longer than the 15 inline bytes";

        fn searched(platform: &Arc<TestPlatform>) -> DbImpl<FileStorageImpl<TestPlatform>> {
            let mut db = open(file(platform), StorageConfig::default());
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
            }
        }

        mod exec_mut {
            use super::*;

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
                    let mut db = searched(&platform);
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
        }
    }
}
