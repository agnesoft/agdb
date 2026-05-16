use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::DbSerialize;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, DbSerialize)]
pub(crate) struct SaveUserToken {
    pub(crate) user: String,
    pub(crate) new_token: String,
    pub(crate) agent: String,
    pub(crate) session: String,
}

impl Action for SaveUserToken {
    async fn exec(self, db: ServerDb, _db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        let user_id = db.user_id(&self.user).await?;
        db.save_token(user_id, &self.new_token, self.agent, self.session)
            .await?;

        Ok(ClusterActionResult::None)
    }
}
