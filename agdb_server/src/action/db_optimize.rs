use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::AgdbDeSerialize;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, AgdbDeSerialize)]
pub(crate) struct DbOptimize {
    pub(crate) owner: String,
    pub(crate) db: String,
}

impl Action for DbOptimize {
    async fn exec(self, _db: ServerDb, db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        db_pool.optimize_db(&self.owner, &self.db).await?;

        Ok(ClusterActionResult::None)
    }
}
