use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::DbSerialize;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, DbSerialize)]
pub(crate) struct RemoveUserSession {
    pub(crate) session: String,
}

impl Action for RemoveUserSession {
    async fn exec(self, db: ServerDb, _db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        db.remove_session(self.session).await?;

        Ok(ClusterActionResult::None)
    }
}
