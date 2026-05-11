use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::DbSerialize;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, DbSerialize)]
pub(crate) struct ClusterLogout {
    pub(crate) user: String,
}

impl Action for ClusterLogout {
    async fn exec(self, db: ServerDb, _db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        if self.user.is_empty() {
            db.remove_all_tokens().await?;
        } else {
            let user_id = db.user_id(&self.user).await?;
            db.remove_tokens(user_id).await?;
        }

        Ok(ClusterActionResult::None)
    }
}
