pub(crate) mod change_password;
pub(crate) mod cluster_login;
pub(crate) mod user_add;
pub(crate) mod user_remove;

use crate::action::change_password::ChangePassword;
use crate::action::cluster_login::ClusterLogin;
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
}

pub(crate) trait Action: Sized {
    async fn exec(self, db: &mut ServerDb, db_pool: &mut DbPool, config: &Config) -> ServerResult;
}

impl ClusterAction {
    pub(crate) async fn exec(
        self,
        db: &mut ServerDb,
        db_pool: &mut DbPool,
        config: &Config,
    ) -> ServerResult<()> {
        match self {
            ClusterAction::UserAdd(action) => action.exec(db, db_pool, config).await,
            ClusterAction::ClusterLogin(action) => action.exec(db, db_pool, config).await,
            ClusterAction::ChangePassword(action) => action.exec(db, db_pool, config).await,
            ClusterAction::UserRemove(action) => action.exec(db, db_pool, config).await,
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
