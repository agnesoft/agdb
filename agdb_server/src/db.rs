use agdb::DbError;
use agdb::DbId;
use agdb::DbImpl;
use agdb::FileStorage;
use agdb::FileStorageMemoryMapped;
use agdb::MemoryStorage;
use agdb::QueryBuilder;
use agdb::QueryId;
use agdb::StorageData;
use agdb::StorageSlice;
use agdb::UserValue;
use anyhow::anyhow;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::PoisonError;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;

const SERVER_DB_NAME: &str = "mapped:agdb_server.agdb";

enum ServerDbStorage {
    MemoryMapped(FileStorageMemoryMapped),
    Memory(MemoryStorage),
    File(FileStorage),
}

impl StorageData for ServerDbStorage {
    fn backup(&mut self, name: &str) -> Result<(), DbError> {
        match self {
            ServerDbStorage::MemoryMapped(s) => s.backup(name),
            ServerDbStorage::Memory(s) => s.backup(name),
            ServerDbStorage::File(s) => s.backup(name),
        }
    }

    fn flush(&mut self) -> Result<(), DbError> {
        match self {
            ServerDbStorage::MemoryMapped(s) => s.flush(),
            ServerDbStorage::Memory(s) => s.flush(),
            ServerDbStorage::File(s) => s.flush(),
        }
    }

    fn len(&self) -> u64 {
        match self {
            ServerDbStorage::MemoryMapped(s) => s.len(),
            ServerDbStorage::Memory(s) => s.len(),
            ServerDbStorage::File(s) => s.len(),
        }
    }

    fn name(&self) -> &str {
        match self {
            ServerDbStorage::MemoryMapped(s) => s.name(),
            ServerDbStorage::Memory(s) => s.name(),
            ServerDbStorage::File(s) => s.name(),
        }
    }

    fn new(name: &str) -> Result<Self, DbError> {
        let (db_type, name) = name.split_once(':').ok_or(DbError::from("Invalid server database name format, must be 'type:name'. Allowed types: mapped, memory, file."))?;

        match db_type {
            "mapped" => Ok(Self::MemoryMapped(FileStorageMemoryMapped::new(name)?)),
            "memory" => Ok(Self::Memory(MemoryStorage::new(name)?)),
            "file" => Ok(Self::File(FileStorage::new(name)?)),
            _ => Err(DbError::from(format!(
                "Invalid db type '{}', must be one of 'mapped', 'memory', 'file'.",
                db_type
            ))),
        }
    }

    fn read(&self, pos: u64, value_len: u64) -> Result<StorageSlice, DbError> {
        match self {
            ServerDbStorage::MemoryMapped(s) => s.read(pos, value_len),
            ServerDbStorage::Memory(s) => s.read(pos, value_len),
            ServerDbStorage::File(s) => s.read(pos, value_len),
        }
    }

    fn resize(&mut self, new_len: u64) -> Result<(), DbError> {
        match self {
            ServerDbStorage::MemoryMapped(s) => s.resize(new_len),
            ServerDbStorage::Memory(s) => s.resize(new_len),
            ServerDbStorage::File(s) => s.resize(new_len),
        }
    }

    fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError> {
        match self {
            ServerDbStorage::MemoryMapped(s) => s.write(pos, bytes),
            ServerDbStorage::Memory(s) => s.write(pos, bytes),
            ServerDbStorage::File(s) => s.write(pos, bytes),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            ServerDbStorage::MemoryMapped(s) => s.is_empty(),
            ServerDbStorage::Memory(s) => s.is_empty(),
            ServerDbStorage::File(s) => s.is_empty(),
        }
    }
}

type ServerDbImpl = DbImpl<ServerDbStorage>;
pub(crate) struct ServerDb(Arc<RwLock<ServerDbImpl>>);

#[allow(dead_code)]
pub(crate) struct DbPoolImpl {
    pub(crate) server_db: ServerDb,
    pub(crate) pool: RwLock<HashMap<String, ServerDb>>,
}

#[derive(UserValue)]
pub(crate) struct User {
    pub(crate) db_id: Option<DbId>,
    pub(crate) name: String,
    pub(crate) password: Vec<u8>,
    pub(crate) salt: Vec<u8>,
    pub(crate) token: String,
}

#[derive(UserValue)]
pub(crate) struct Database {
    pub(crate) db_id: Option<DbId>,
    pub(crate) name: String,
    pub(crate) db_type: String,
}

#[derive(Clone)]
pub(crate) struct DbPool(pub(crate) Arc<DbPoolImpl>);

impl DbPool {
    pub(crate) fn new() -> anyhow::Result<Self> {
        let db_pool = Self(Arc::new(DbPoolImpl {
            server_db: ServerDb::new(SERVER_DB_NAME)?,
            pool: RwLock::new(HashMap::new()),
        }));

        db_pool.0.server_db.get_mut()?.exec_mut(
            &QueryBuilder::insert()
                .nodes()
                .aliases(vec!["users", "dbs"])
                .query(),
        )?;

        Ok(db_pool)
    }

    pub(crate) fn add_database(&self, user: DbId, database: Database) -> anyhow::Result<()> {
        let db = ServerDb::new(&format!("{}:{}", database.db_type, database.name))?;
        self.get_pool_mut()?.insert(database.name.clone(), db);

        self.0.server_db.get_mut()?.transaction_mut(|t| {
            let db = t.exec_mut(&QueryBuilder::insert().nodes().values(&database).query())?;

            t.exec_mut(
                &QueryBuilder::insert()
                    .edges()
                    .from(vec![QueryId::from(user), "dbs".into()])
                    .to(db)
                    .values(vec![vec![("owner", 1).into()], vec![]])
                    .query(),
            )
        })?;
        Ok(())
    }

    pub(crate) fn create_user(&self, user: User) -> anyhow::Result<()> {
        self.0.server_db.get_mut()?.transaction_mut(|t| {
            let user = t.exec_mut(&QueryBuilder::insert().nodes().values(&user).query())?;

            t.exec_mut(
                &QueryBuilder::insert()
                    .edges()
                    .from("users")
                    .to(user)
                    .query(),
            )
        })?;
        Ok(())
    }

    pub(crate) fn delete_database(&self, db: Database) -> anyhow::Result<()> {
        let filename = self.remove_database(db)?.get()?.filename().to_string();
        let path = Path::new(&filename);

        if path.exists() {
            std::fs::remove_file(&filename)?;
            let dot_file = path
                .parent()
                .unwrap_or(Path::new("./"))
                .join(format!(".{filename}"));
            std::fs::remove_file(dot_file)?;
        }

        Ok(())
    }

    pub(crate) fn find_database(&self, name: &str) -> anyhow::Result<Database> {
        Ok(self
            .0
            .server_db
            .get()?
            .exec(
                &QueryBuilder::select()
                    .ids(
                        QueryBuilder::search()
                            .from("dbs")
                            .limit(1)
                            .where_()
                            .distance(agdb::CountComparison::Equal(2))
                            .and()
                            .key("name")
                            .value(agdb::Comparison::Equal(name.into()))
                            .query(),
                    )
                    .query(),
            )?
            .try_into()?)
    }

    pub(crate) fn find_user_databases(&self, user: DbId) -> anyhow::Result<Vec<Database>> {
        Ok(self
            .0
            .server_db
            .get()?
            .exec(
                &QueryBuilder::select()
                    .ids(
                        QueryBuilder::search()
                            .from(user)
                            .where_()
                            .distance(agdb::CountComparison::Equal(2))
                            .query(),
                    )
                    .query(),
            )?
            .try_into()?)
    }

    pub(crate) fn find_user_database(&self, user: DbId, name: &str) -> anyhow::Result<Database> {
        Ok(self
            .0
            .server_db
            .get()?
            .exec(
                &QueryBuilder::select()
                    .ids(
                        QueryBuilder::search()
                            .from(user)
                            .where_()
                            .distance(agdb::CountComparison::Equal(2))
                            .and()
                            .key("name")
                            .value(agdb::Comparison::Equal(name.into()))
                            .query(),
                    )
                    .query(),
            )?
            .try_into()?)
    }

    pub(crate) fn find_user(&self, name: &str) -> anyhow::Result<User> {
        Ok(self
            .0
            .server_db
            .get()?
            .exec(
                &QueryBuilder::select()
                    .ids(
                        QueryBuilder::search()
                            .from("users")
                            .limit(1)
                            .where_()
                            .distance(agdb::CountComparison::Equal(2))
                            .and()
                            .key("name")
                            .value(agdb::Comparison::Equal(name.into()))
                            .query(),
                    )
                    .query(),
            )?
            .try_into()?)
    }

    pub(crate) fn find_user_id(&self, token: &str) -> anyhow::Result<DbId> {
        Ok(self
            .0
            .server_db
            .get()?
            .exec(
                &QueryBuilder::search()
                    .from("users")
                    .limit(1)
                    .where_()
                    .distance(agdb::CountComparison::Equal(2))
                    .and()
                    .key("token")
                    .value(agdb::Comparison::Equal(token.into()))
                    .query(),
            )?
            .elements
            .get(0)
            .ok_or(anyhow!("No user found for token '{token}'"))?
            .id)
    }

    pub(crate) fn remove_database(&self, db: Database) -> anyhow::Result<ServerDb> {
        self.0
            .server_db
            .get_mut()?
            .exec_mut(&QueryBuilder::remove().ids(db.db_id.unwrap()).query())?;

        Ok(self.get_pool_mut()?.remove(&db.name).unwrap())
    }

    pub(crate) fn save_token(&self, user: DbId, token: &str) -> anyhow::Result<()> {
        self.0.server_db.get_mut()?.exec_mut(
            &QueryBuilder::insert()
                .values_uniform(vec![("token", token).into()])
                .ids(user)
                .query(),
        )?;
        Ok(())
    }

    pub(crate) fn save_user(&self, user: User) -> anyhow::Result<()> {
        self.0
            .server_db
            .get_mut()?
            .exec_mut(&QueryBuilder::insert().element(&user).query())?;
        Ok(())
    }

    // fn get_pool(&self) -> anyhow::Result<RwLockReadGuard<HashMap<String, ServerDb>>> {
    //     self.0.pool.read().map_err(map_error)
    // }

    fn get_pool_mut(&self) -> anyhow::Result<RwLockWriteGuard<HashMap<String, ServerDb>>> {
        self.0.pool.write().map_err(map_error)
    }
}

impl ServerDb {
    pub(crate) fn new(name: &str) -> anyhow::Result<Self> {
        Ok(Self(Arc::new(RwLock::new(ServerDbImpl::new(name)?))))
    }

    fn get(&self) -> anyhow::Result<RwLockReadGuard<ServerDbImpl>> {
        self.0.read().map_err(map_error)
    }

    fn get_mut(&self) -> anyhow::Result<RwLockWriteGuard<ServerDbImpl>> {
        self.0.write().map_err(map_error)
    }
}

fn map_error<T>(e: PoisonError<T>) -> anyhow::Error {
    anyhow!(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestFile(String);

    impl TestFile {
        fn new(name: &str) -> Self {
            let _ = std::fs::remove_file(name);
            Self(name.to_string())
        }
    }

    impl Drop for TestFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    impl std::fmt::Debug for ServerDbStorage {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::MemoryMapped(_) => f.write_str("MemoryMapped"),
                Self::Memory(_) => f.write_str("Memory"),
                Self::File(_) => f.write_str("File"),
            }
        }
    }

    #[test]
    fn map_error_test() {
        let error = PoisonError::new("guard");
        map_error(error);
    }

    #[test]
    fn invalid_db_name() {
        assert_eq!(
            ServerDbStorage::new("db.agdb").unwrap_err().description,
            "Invalid server database name format, must be 'type:name'. Allowed types: mapped, memory, file."
        );
    }

    #[test]
    fn invalid_db_type() {
        assert_eq!(
            ServerDbStorage::new("sometype:db.agdb")
                .unwrap_err()
                .description,
            "Invalid db type 'sometype', must be one of 'mapped', 'memory', 'file'."
        );
    }

    #[test]
    fn file_storage() -> anyhow::Result<()> {
        let test_file = TestFile::new("file_storage.agdb");
        let _test_file_dot = TestFile::new(".file_storage.agdb");
        let test_file2 = TestFile::new("file_storage_backup.agdb");
        let mut storage = ServerDbStorage::new(&format!("file:{}", test_file.0))?;
        println!("{:?}", storage);
        storage.backup(&test_file2.0)?;
        assert!(std::path::Path::new(&test_file2.0).exists());
        storage.flush()?;
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
        assert_eq!(storage.name(), test_file.0);
        assert!(storage.read(0, 0)?.is_empty());
        storage.write(0, &[])?;
        storage.resize(0)?;
        Ok(())
    }

    #[test]
    fn mapped_storage() -> anyhow::Result<()> {
        let test_file = TestFile::new("mapped_storage.agdb");
        let _test_file_dot = TestFile::new(".mapped_storage.agdb");
        let test_file2 = TestFile::new("mapped_storage_backup.agdb");
        let mut storage = ServerDbStorage::new(&format!("mapped:{}", test_file.0))?;
        println!("{:?}", storage);
        storage.backup(&test_file2.0)?;
        assert!(std::path::Path::new(&test_file2.0).exists());
        storage.flush()?;
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
        assert_eq!(storage.name(), test_file.0);
        assert!(storage.read(0, 0)?.is_empty());
        storage.write(0, &[])?;
        storage.resize(0)?;
        Ok(())
    }

    #[test]
    fn memory_storage() -> anyhow::Result<()> {
        let mut storage = ServerDbStorage::new("memory:db_test.agdb")?;
        println!("{:?}", storage);
        storage.backup("backup_test")?;
        storage.flush()?;
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
        assert_eq!(storage.name(), "db_test.agdb");
        assert!(storage.read(0, 0)?.is_empty());
        storage.write(0, &[])?;
        storage.resize(0)?;
        Ok(())
    }
}
