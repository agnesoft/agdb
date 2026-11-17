use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::DbSerialize;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, DbSerialize)]
pub(crate) struct DashboardRemove {
    pub(crate) owner: String,
    pub(crate) name: String,
}

impl Action for DashboardRemove {
    async fn exec(self, db: ServerDb, _db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        db.remove_dashboard(&self.owner, &self.name).await?;
        Ok(ClusterActionResult::None)
    }
}
