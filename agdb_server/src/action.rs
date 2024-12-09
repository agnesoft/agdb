pub(crate) mod login;
pub(crate) mod user_add;

use crate::action::user_add::UserAdd;
use crate::db_pool::DbPool;
use crate::server_db::ServerDb;
use crate::server_error::ServerResult;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) enum ClusterAction {
    UserAdd(user_add::UserAdd),
    Login(login::Login),
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
            ClusterAction::Login(action) => action.exec(db, db_pool).await,
        }
    }
}

impl From<UserAdd> for ClusterAction {
    fn from(value: UserAdd) -> Self {
        ClusterAction::UserAdd(value)
    }
}
