pub(crate) mod change_password;
pub(crate) mod cluster_login;
pub(crate) mod db_add;
pub(crate) mod db_backup;
pub(crate) mod db_clear;
pub(crate) mod db_convert;
pub(crate) mod user_add;
pub(crate) mod user_remove;

use crate::action::change_password::ChangePassword;
use crate::action::cluster_login::ClusterLogin;
use crate::action::db_add::DbAdd;
use crate::action::db_backup::DbBackup;
use crate::action::db_clear::DbClear;
use crate::action::db_convert::DbConvert;
use crate::action::user_add::UserAdd;
use crate::action::user_remove::UserRemove;
use crate::config::Config;
use crate::db_pool::DbPool;
use crate::server_db::ServerDb;
use crate::server_error::ServerResult;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) enum ClusterAction {
    UserAdd(UserAdd),
    ClusterLogin(ClusterLogin),
    ChangePassword(ChangePassword),
    UserRemove(UserRemove),
    DbAdd(DbAdd),
    DbBackup(DbBackup),
    DbClear(DbClear),
    DbConvert(DbConvert),
}

pub(crate) trait Action: Sized {
    async fn exec(self, db: ServerDb, db_pool: DbPool, config: &Config) -> ServerResult;
}

impl ClusterAction {
    pub(crate) async fn exec(
        self,
        db: ServerDb,
        db_pool: DbPool,
        config: &Config,
    ) -> ServerResult<()> {
        match self {
            ClusterAction::UserAdd(action) => action.exec(db, db_pool, config).await,
            ClusterAction::ClusterLogin(action) => action.exec(db, db_pool, config).await,
            ClusterAction::ChangePassword(action) => action.exec(db, db_pool, config).await,
            ClusterAction::UserRemove(action) => action.exec(db, db_pool, config).await,
            ClusterAction::DbAdd(action) => action.exec(db, db_pool, config).await,
            ClusterAction::DbBackup(db_backup) => db_backup.exec(db, db_pool, config).await,
            ClusterAction::DbClear(db_clear) => db_clear.exec(db, db_pool, config).await,
            ClusterAction::DbConvert(db_convert) => db_convert.exec(db, db_pool, config).await,
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

impl From<ChangePassword> for ClusterAction {
    fn from(value: ChangePassword) -> Self {
        ClusterAction::ChangePassword(value)
    }
}

impl From<UserRemove> for ClusterAction {
    fn from(value: UserRemove) -> Self {
        ClusterAction::UserRemove(value)
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
