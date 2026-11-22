use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::DbSerialize;
use serde::Deserialize;
use serde::Serialize;

/// Sets or, with `None`, clears the dashboard the `user` opens by default.
#[derive(Clone, Serialize, Deserialize, DbSerialize)]
pub(crate) struct DashboardHomeSet {
    pub(crate) user: String,
    /// `(owner, name)` of the dashboard.
    pub(crate) dashboard: Option<(String, String)>,
}

impl Action for DashboardHomeSet {
    async fn exec(self, db: ServerDb, _db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        let user_id = db.user_id(&self.user).await?;
        let dashboard = match &self.dashboard {
            Some((owner, name)) => {
                db.user_dashboard(user_id, owner, name)
                    .await?
                    .dashboard
                    .db_id
            }
            None => None,
        };
        db.set_home_dashboard(user_id, dashboard).await?;

        Ok(ClusterActionResult::None)
    }
}
