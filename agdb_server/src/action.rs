pub(crate) mod change_password;
pub(crate) mod cluster_login;
pub(crate) mod cluster_logout;
pub(crate) mod db_add;
pub(crate) mod db_backup;
pub(crate) mod db_clear;
pub(crate) mod db_convert;
pub(crate) mod db_copy;
pub(crate) mod db_delete;
pub(crate) mod db_exec;
pub(crate) mod db_optimize;
pub(crate) mod db_remove;
pub(crate) mod db_rename;
pub(crate) mod db_restore;
pub(crate) mod db_user_add;
pub(crate) mod db_user_remove;
pub(crate) mod user_add;
pub(crate) mod user_delete;

use crate::action::change_password::ChangePassword;
use crate::action::cluster_login::ClusterLogin;
use crate::action::cluster_logout::ClusterLogout;
use crate::action::db_add::DbAdd;
use crate::action::db_backup::DbBackup;
use crate::action::db_clear::DbClear;
use crate::action::db_convert::DbConvert;
use crate::action::db_copy::DbCopy;
use crate::action::db_delete::DbDelete;
use crate::action::db_exec::DbExec;
use crate::action::db_optimize::DbOptimize;
use crate::action::db_remove::DbRemove;
use crate::action::db_rename::DbRename;
use crate::action::db_restore::DbRestore;
use crate::action::db_user_add::DbUserAdd;
use crate::action::db_user_remove::DbUserRemove;
use crate::action::user_add::UserAdd;
use crate::action::user_delete::UserDelete;
use crate::db_pool::DbPool;
use crate::server_db::ServerDb;
use crate::server_error::ServerResult;
use agdb::DbSerialize;
use agdb::QueryResult;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, DbSerialize)]
pub(crate) enum ClusterAction {
    UserAdd(UserAdd),
    ClusterLogin(ClusterLogin),
    ClusterLogout(ClusterLogout),
    ChangePassword(ChangePassword),
    UserDelete(UserDelete),
    DbAdd(DbAdd),
    DbBackup(DbBackup),
    DbClear(DbClear),
    DbConvert(DbConvert),
    DbCopy(DbCopy),
    DbDelete(DbDelete),
    DbRemove(DbRemove),
    DbExec(DbExec),
    DbOptimize(DbOptimize),
    DbRestore(DbRestore),
    DbRename(DbRename),
    DbUserAdd(DbUserAdd),
    DbUserRemove(DbUserRemove),
}

pub(crate) enum ClusterActionResult {
    None,
    QueryResults(Vec<QueryResult>),
}

pub(crate) trait Action: Sized {
    async fn exec(self, db: ServerDb, db_pool: DbPool) -> ServerResult<ClusterActionResult>;
}

impl ClusterAction {
    pub(crate) async fn exec(
        self,
        db: ServerDb,
        db_pool: DbPool,
    ) -> ServerResult<ClusterActionResult> {
        match self {
            ClusterAction::UserAdd(action) => action.exec(db, db_pool).await,
            ClusterAction::ClusterLogin(action) => action.exec(db, db_pool).await,
            ClusterAction::ClusterLogout(action) => action.exec(db, db_pool).await,
            ClusterAction::ChangePassword(action) => action.exec(db, db_pool).await,
            ClusterAction::UserDelete(action) => action.exec(db, db_pool).await,
            ClusterAction::DbAdd(action) => action.exec(db, db_pool).await,
            ClusterAction::DbBackup(action) => action.exec(db, db_pool).await,
            ClusterAction::DbClear(action) => action.exec(db, db_pool).await,
            ClusterAction::DbConvert(action) => action.exec(db, db_pool).await,
            ClusterAction::DbCopy(action) => action.exec(db, db_pool).await,
            ClusterAction::DbDelete(action) => action.exec(db, db_pool).await,
            ClusterAction::DbRemove(action) => action.exec(db, db_pool).await,
            ClusterAction::DbExec(action) => action.exec(db, db_pool).await,
            ClusterAction::DbOptimize(action) => action.exec(db, db_pool).await,
            ClusterAction::DbRestore(action) => action.exec(db, db_pool).await,
            ClusterAction::DbRename(action) => action.exec(db, db_pool).await,
            ClusterAction::DbUserAdd(action) => action.exec(db, db_pool).await,
            ClusterAction::DbUserRemove(action) => action.exec(db, db_pool).await,
        }
    }
}

impl From<UserAdd> for ClusterAction {
    fn from(value: UserAdd) -> Self {
        ClusterAction::UserAdd(value)
    }
}

impl From<ClusterLogin> for ClusterAction {
    fn from(value: ClusterLogin) -> Self {
        ClusterAction::ClusterLogin(value)
    }
}

impl From<ClusterLogout> for ClusterAction {
    fn from(value: ClusterLogout) -> Self {
        ClusterAction::ClusterLogout(value)
    }
}

impl From<ChangePassword> for ClusterAction {
    fn from(value: ChangePassword) -> Self {
        ClusterAction::ChangePassword(value)
    }
}

impl From<UserDelete> for ClusterAction {
    fn from(value: UserDelete) -> Self {
        ClusterAction::UserDelete(value)
    }
}

impl From<DbAdd> for ClusterAction {
    fn from(value: DbAdd) -> Self {
        ClusterAction::DbAdd(value)
    }
}

impl From<DbBackup> for ClusterAction {
    fn from(value: DbBackup) -> Self {
        ClusterAction::DbBackup(value)
    }
}

impl From<DbClear> for ClusterAction {
    fn from(value: DbClear) -> Self {
        ClusterAction::DbClear(value)
    }
}

impl From<DbConvert> for ClusterAction {
    fn from(value: DbConvert) -> Self {
        ClusterAction::DbConvert(value)
    }
}

impl From<DbCopy> for ClusterAction {
    fn from(value: DbCopy) -> Self {
        ClusterAction::DbCopy(value)
    }
}

impl From<DbDelete> for ClusterAction {
    fn from(value: DbDelete) -> Self {
        ClusterAction::DbDelete(value)
    }
}

impl From<DbRemove> for ClusterAction {
    fn from(value: DbRemove) -> Self {
        ClusterAction::DbRemove(value)
    }
}

impl From<DbExec> for ClusterAction {
    fn from(value: DbExec) -> Self {
        ClusterAction::DbExec(value)
    }
}

impl From<DbOptimize> for ClusterAction {
    fn from(value: DbOptimize) -> Self {
        ClusterAction::DbOptimize(value)
    }
}

impl From<DbRestore> for ClusterAction {
    fn from(value: DbRestore) -> Self {
        ClusterAction::DbRestore(value)
    }
}

impl From<DbRename> for ClusterAction {
    fn from(value: DbRename) -> Self {
        ClusterAction::DbRename(value)
    }
}

impl From<DbUserAdd> for ClusterAction {
    fn from(value: DbUserAdd) -> Self {
        ClusterAction::DbUserAdd(value)
    }
}

impl From<DbUserRemove> for ClusterAction {
    fn from(value: DbUserRemove) -> Self {
        ClusterAction::DbUserRemove(value)
    }
}

impl Serialize for ClusterAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&agdb::AgdbSerialize::serialize(self))
    }
}

impl<'de> Deserialize<'de> for ClusterAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        agdb::AgdbSerialize::deserialize(&bytes).map_err(serde::de::Error::custom)
    }
}
