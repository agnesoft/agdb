use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterResponse;
use crate::server_error::ServerResult;
use agdb::UserValue;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, UserValue)]
pub(crate) struct Login {
    pub(crate) username: String,
    pub(crate) token: String,
}

impl Action for Login {
    async fn exec(self, db: &mut ServerDb, _db_pool: &mut DbPool) -> ServerResult<ClusterResponse> {
        let user_id = db.user_id(&self.username).await?;
        db.save_token(user_id, &self.token).await?;
        Ok(ClusterResponse::None)
    }
}
