use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::DbSerialize;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, DbSerialize)]
pub(crate) struct DbOptimize {
    pub(crate) owner: String,
    pub(crate) db: String,
    pub(crate) shrink_to_fit: bool,
}

impl Action for DbOptimize {
    async fn exec(self, _db: ServerDb, db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        if self.shrink_to_fit {
            db_pool.shrink_to_fit_db(&self.owner, &self.db).await?;
        } else {
            db_pool.optimize_db(&self.owner, &self.db).await?;
        }

        Ok(ClusterActionResult::None)
    }
}
