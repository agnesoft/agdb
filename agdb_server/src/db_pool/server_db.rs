use crate::db_pool::user_db_storage::UserDbStorage;
use crate::db_pool::Database;
use crate::db_pool::ServerUser;
use crate::server_error::ServerResult;
use agdb::CountComparison;
use agdb::DbId;
use agdb::DbImpl;
use agdb::QueryBuilder;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::RwLockReadGuard;
use tokio::sync::RwLockWriteGuard;

pub(crate) type ServerDbImpl = DbImpl<UserDbStorage>;
pub(crate) struct ServerDb(pub(crate) Arc<RwLock<ServerDbImpl>>);

const DBS: &str = "dbs";
const USERS: &str = "users";
const USERNAME: &str = "username";
const TOKEN: &str = "token";

impl ServerDb {
    pub(crate) async fn get(&self) -> RwLockReadGuard<ServerDbImpl> {
        self.0.read().await
    }

    pub(crate) async fn get_mut(&self) -> RwLockWriteGuard<ServerDbImpl> {
        self.0.write().await
    }

    pub(crate) fn load(name: &str) -> ServerResult<Self> {
        Ok(Self(Arc::new(RwLock::new(ServerDbImpl::new(name)?))))
    }

    pub(crate) fn new(name: &str, admin: ServerUser) -> ServerResult<Self> {
        let mut db = ServerDbImpl::new(name)?;

        db.transaction_mut(|t| {
            t.exec_mut(QueryBuilder::insert().index(USERNAME).query())?;
            t.exec_mut(QueryBuilder::insert().index(TOKEN).query())?;
            t.exec_mut(QueryBuilder::insert().nodes().aliases([USERS, DBS]).query())?;
            let id = t
                .exec_mut(QueryBuilder::insert().element(&admin).query())?
                .elements[0]
                .id;
            t.exec_mut(QueryBuilder::insert().edges().from(USERS).to(id).query())
        })?;

        Ok(Self(Arc::new(RwLock::new(db))))
    }

    pub(crate) async fn find_user_id(&self, username: &str) -> ServerResult<Option<DbId>> {
        Ok(self
            .0
            .read()
            .await
            .exec(
                QueryBuilder::search()
                    .index(USERNAME)
                    .value(username)
                    .query(),
            )?
            .elements
            .first()
            .map(|e| e.id))
    }

    pub(crate) async fn insert_user(&self, user: ServerUser) -> ServerResult<DbId> {
        self.0.write().await.transaction_mut(|t| {
            let id = t
                .exec_mut(QueryBuilder::insert().element(&user).query())?
                .elements[0]
                .id;
            t.exec_mut(QueryBuilder::insert().edges().from(USERS).to(id).query())?;
            Ok(id)
        })
    }

    pub(crate) async fn dbs(&self) -> ServerResult<Vec<Database>> {
        Ok(self
            .0
            .read()
            .await
            .exec(
                QueryBuilder::select()
                    .elements::<Database>()
                    .search()
                    .from(DBS)
                    .where_()
                    .distance(CountComparison::Equal(2))
                    .query(),
            )?
            .try_into()?)
    }
}
