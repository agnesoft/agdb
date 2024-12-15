use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterResponse;
use crate::config::Config;
use crate::server_error::ServerResult;
use agdb::UserValue;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, UserValue)]
pub(crate) struct UserRemove {
    pub(crate) user: String,
}

impl Action for UserRemove {
    async fn exec(
        self,
        db: &mut ServerDb,
        db_pool: &mut DbPool,
        config: &Config,
    ) -> ServerResult<ClusterResponse> {
        let dbs = db.remove_user(&self.user).await?;
        db_pool.remove_user_dbs(&self.user, &dbs, config).await?;

        Ok(ClusterResponse::None)
    }
}
