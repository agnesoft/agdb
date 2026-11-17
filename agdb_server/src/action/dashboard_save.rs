use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_db::ServerDashboard;
use crate::server_error::ServerResult;
use agdb::DbSerialize;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, DbSerialize)]
pub(crate) struct DashboardSave {
    pub(crate) owner: String,
    pub(crate) name: String,
    pub(crate) config: String,
    /// Fails when the dashboard exists (add), instead of replacing it (save).
    pub(crate) create: bool,
    /// Set by the leader so every node stores the same value.
    pub(crate) updated: u64,
}

impl Action for DashboardSave {
    async fn exec(self, db: ServerDb, _db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        db.save_dashboard(
            ServerDashboard {
                db_id: None,
                owner: self.owner,
                name: self.name,
                config: self.config,
                updated: self.updated,
            },
            self.create,
        )
        .await?;
        Ok(ClusterActionResult::None)
    }
}
