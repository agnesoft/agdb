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
    pub(crate) user: String,
    pub(crate) session: i64,
}

impl Action for RemoveUserSession {
    async fn exec(self, db: ServerDb, _db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        let user_id = db.user_id(&self.user).await?;
        db.remove_session(user_id, self.session).await?;

        Ok(ClusterActionResult::None)
    }
}
