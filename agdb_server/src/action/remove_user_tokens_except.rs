use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::DbSerialize;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, DbSerialize)]
pub(crate) struct RemoveUserTokensExcept {
    pub(crate) user: String,
    pub(crate) token: String,
}

impl Action for RemoveUserTokensExcept {
    async fn exec(self, db: ServerDb, _db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        let user_id = db.user_id(&self.user).await?;
        db.remove_tokens_except(user_id, &self.token).await?;

        Ok(ClusterActionResult::None)
    }
}
