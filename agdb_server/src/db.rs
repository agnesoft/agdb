use agdb::Db;
use agdb::DbFile;
use agdb::DbMemory;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

const SERVER_DB_NAME: &str = "agdb_studio.agdb";

#[allow(dead_code)]
pub(crate) enum DbType {
    MemoryMapped(Db),
    Memory(DbMemory),
    File(DbFile),
}

#[allow(dead_code)]
pub(crate) struct ServerDb {
    pub(crate) db: Arc<RwLock<DbType>>,
}

#[allow(dead_code)]
pub(crate) struct DbPoolImpl {
    pub(crate) db: ServerDb,
    pub(crate) pool: HashMap<String, ServerDb>,
}

#[derive(Clone)]
pub(crate) struct DbPool(pub(crate) Arc<RwLock<DbPoolImpl>>);

impl DbPool {
    pub(crate) fn new() -> anyhow::Result<Self> {
        Ok(Self(Arc::new(RwLock::new(DbPoolImpl {
            db: ServerDb {
                db: Arc::new(RwLock::new(DbType::MemoryMapped(Db::new(SERVER_DB_NAME)?))),
            },
            pool: HashMap::new(),
        }))))
    }
}
