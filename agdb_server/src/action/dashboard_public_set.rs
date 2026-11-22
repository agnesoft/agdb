use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_db::dashboard_not_found;
use crate::server_error::ServerResult;
use agdb::DbSerialize;
use serde::Deserialize;
use serde::Serialize;

/// Makes a dashboard public (viewable by every user) or private.
#[derive(Clone, Serialize, Deserialize, DbSerialize)]
pub(crate) struct DashboardPublicSet {
    pub(crate) owner: String,
    pub(crate) name: String,
    pub(crate) public: bool,
}

impl Action for DashboardPublicSet {
    async fn exec(self, db: ServerDb, _db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        let dashboard = db.dashboard(&self.owner, &self.name).await?;
        db.set_dashboard_public(
            dashboard
                .db_id
                .ok_or_else(|| dashboard_not_found(&self.name))?,
            self.public,
        )
        .await?;

        Ok(ClusterActionResult::None)
    }
}
