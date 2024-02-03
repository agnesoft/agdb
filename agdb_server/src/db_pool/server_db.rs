use crate::db_pool::server_db_storage::ServerDbStorage;
use crate::server_error::ServerResult;
use agdb::DbImpl;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::RwLockReadGuard;
use tokio::sync::RwLockWriteGuard;

pub(crate) type ServerDbImpl = DbImpl<ServerDbStorage>;
pub(crate) struct ServerDb(pub(crate) Arc<RwLock<ServerDbImpl>>);

impl ServerDb {
    pub(crate) fn new(name: &str) -> ServerResult<Self> {
        Ok(Self(Arc::new(RwLock::new(ServerDbImpl::new(name)?))))
    }

    pub(crate) async fn copy(&self, name: &str) -> ServerResult<Self> {
        Ok(Self(Arc::new(RwLock::new(self.get().await.copy(name)?))))
    }

    pub(crate) async fn get(&self) -> RwLockReadGuard<ServerDbImpl> {
        self.0.read().await
    }

    pub(crate) async fn get_mut(&self) -> RwLockWriteGuard<ServerDbImpl> {
        self.0.write().await
    }
}
