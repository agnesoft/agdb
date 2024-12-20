use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::action::Config;
use crate::server_error::ServerResult;
use crate::utilities::db_name;
use agdb::UserValue;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, UserValue)]
pub(crate) struct DbOptimize {
    pub(crate) owner: String,
    pub(crate) db: String,
}

impl Action for DbOptimize {
    async fn exec(
        self,
        _db: ServerDb,
        db_pool: DbPool,
        _config: &Config,
    ) -> ServerResult<ClusterActionResult> {
        let name = db_name(&self.owner, &self.db);
        db_pool.optimize_db(&name).await?;

        Ok(ClusterActionResult::None)
    }
}
