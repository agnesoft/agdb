use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterResponse;
use crate::server_error::ServerResult;
use agdb::UserValue;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, UserValue)]
pub(crate) struct ClusterLogin {
    pub(crate) user: String,
    pub(crate) new_token: String,
}

impl Action for ClusterLogin {
    async fn exec(self, db: &mut ServerDb, _db_pool: &mut DbPool) -> ServerResult<ClusterResponse> {
        let user_id = db.user_id(&self.user).await?;
        db.save_token(user_id, &self.new_token).await?;

        Ok(ClusterResponse::None)
    }
}
