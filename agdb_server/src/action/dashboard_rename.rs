use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::DbSerialize;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, DbSerialize)]
pub(crate) struct DashboardRename {
    pub(crate) owner: String,
    pub(crate) name: String,
    pub(crate) new_name: String,
    /// Set by the leader so every node stores the same value.
    pub(crate) updated: u64,
}

impl Action for DashboardRename {
    async fn exec(self, db: ServerDb, _db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        db.rename_dashboard(&self.owner, &self.name, &self.new_name, self.updated)
            .await?;
        Ok(ClusterActionResult::None)
    }
}
