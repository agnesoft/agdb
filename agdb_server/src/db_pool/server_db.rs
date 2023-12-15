use crate::db_pool::server_db_storage::ServerDbStorage;
use crate::server_error::ServerResult;
use agdb::DbImpl;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;

pub(crate) type ServerDbImpl = DbImpl<ServerDbStorage>;
pub(crate) struct ServerDb(pub(crate) Arc<RwLock<ServerDbImpl>>);

impl ServerDb {
    pub(crate) fn new(name: &str) -> ServerResult<Self> {
        Ok(Self(Arc::new(RwLock::new(ServerDbImpl::new(name)?))))
    }

    pub(crate) fn get(&self) -> ServerResult<RwLockReadGuard<ServerDbImpl>> {
        Ok(self.0.read()?)
    }

    pub(crate) fn get_mut(&self) -> ServerResult<RwLockWriteGuard<ServerDbImpl>> {
        Ok(self.0.write()?)
    }
}
