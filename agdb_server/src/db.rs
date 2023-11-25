use agdb::Db;
use agdb::DbError;
use agdb::DbFile;
use agdb::DbId;
use agdb::DbImpl;
use agdb::DbMemory;
use agdb::Query;
use agdb::QueryBuilder;
use agdb::QueryError;
use agdb::QueryMut;
use agdb::QueryResult;
use agdb::StorageData;
use agdb::TransactionMut;
use agdb::UserValue;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::sync::RwLock;

const SERVER_DB_NAME: &str = "agdb_server.agdb";

#[allow(dead_code)]
pub(crate) enum DbType {
    MemoryMapped(Db),
    Memory(DbMemory),
    File(DbFile),
}

#[allow(dead_code)]
pub(crate) struct ServerDb(pub(crate) Arc<RwLock<DbType>>);

#[allow(dead_code)]
pub(crate) struct DbPoolImpl {
    pub(crate) server_db: ServerDb,
    pub(crate) pool: HashMap<String, ServerDb>,
}

#[derive(UserValue)]
pub(crate) struct User {
    pub(crate) db_id: Option<DbId>,
    pub(crate) name: String,
    pub(crate) password: Vec<u8>,
    pub(crate) salt: Vec<u8>,
    pub(crate) token: String,
}

#[derive(Clone)]
pub(crate) struct DbPool(pub(crate) Arc<DbPoolImpl>);

impl DbPool {
    pub(crate) fn new() -> anyhow::Result<Self> {
        let mut db_pool = Self(Arc::new(DbPoolImpl {
            server_db: ServerDb(Arc::new(RwLock::new(DbType::MemoryMapped(Db::new(
                SERVER_DB_NAME,
            )?)))),
            pool: HashMap::new(),
        }));

        db_pool.0.server_db.exec_mut(
            &QueryBuilder::insert()
                .nodes()
                .aliases(vec!["users", "dbs"])
                .query(),
        )?;

        Ok(db_pool)
    }

    pub(crate) fn find_user(&self, name: &str) -> anyhow::Result<User> {
        Ok(self
            .0
            .server_db
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

    pub(crate) fn create_user(&self, user: User) -> anyhow::Result<()> {
        todo!()
    }
}

impl ServerDb {
    pub(crate) fn db<S: StorageData>(&self) -> &DbImpl<S> {
        let db_type = *self.0.read().unwrap();
        match db_type {
            DbType::MemoryMapped(db) => db,
            DbType::Memory(db) => db,
            DbType::File(db) => db,
        }
    }

    pub(crate) fn exec<T: Query>(&self, query: &T) -> Result<QueryResult, QueryError> {
        let db_type = *self.0.read()?;
        match db_type {
            DbType::MemoryMapped(db) => db.exec(query),
            DbType::Memory(db) => db.exec(query),
            DbType::File(db) => db.exec(query),
        }
    }

    pub(crate) fn exec_mut<T: QueryMut>(&mut self, query: &T) -> Result<QueryResult, QueryError> {
        let mut db_type = *self.0.write()?;
        match db_type {
            DbType::MemoryMapped(mut db) => db.exec_mut(query),
            DbType::Memory(mut db) => db.exec_mut(query),
            DbType::File(mut db) => db.exec_mut(query),
        }
    }
}
