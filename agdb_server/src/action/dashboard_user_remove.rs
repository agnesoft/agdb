use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_db::dashboard_not_found;
use crate::server_error::ServerResult;
use agdb::DbSerialize;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, DbSerialize)]
pub(crate) struct DashboardUserRemove {
    pub(crate) owner: String,
    pub(crate) name: String,
    pub(crate) user: String,
}

impl Action for DashboardUserRemove {
    async fn exec(self, db: ServerDb, _db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        let dashboard = db.dashboard(&self.owner, &self.name).await?;
        let user_id = db.user_id(&self.user).await?;
        db.remove_dashboard_user(
            dashboard
                .db_id
                .ok_or_else(|| dashboard_not_found(&self.name))?,
            user_id,
        )
        .await?;

        Ok(ClusterActionResult::None)
    }
}
