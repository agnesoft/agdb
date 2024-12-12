pub(crate) mod cluster_login;
pub(crate) mod user_add;

use crate::action::cluster_login::ClusterLogin;
use crate::action::user_add::UserAdd;
use crate::db_pool::DbPool;
use crate::server_db::ServerDb;
use crate::server_error::ServerResult;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) enum ClusterAction {
    UserAdd(UserAdd),
    ClusterLogin(ClusterLogin),
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) enum ClusterResponse {
    None,
}

pub(crate) trait Action: Sized {
    async fn exec(self, db: &mut ServerDb, db_pool: &mut DbPool) -> ServerResult<ClusterResponse>;
}

impl Action for ClusterAction {
    async fn exec(self, db: &mut ServerDb, db_pool: &mut DbPool) -> ServerResult<ClusterResponse> {
        match self {
            ClusterAction::UserAdd(action) => action.exec(db, db_pool).await,
            ClusterAction::ClusterLogin(action) => action.exec(db, db_pool).await,
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
