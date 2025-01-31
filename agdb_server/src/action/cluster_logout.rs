use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::AgdbDeSerialize;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, AgdbDeSerialize)]
pub(crate) struct ClusterLogout {}

impl Action for ClusterLogout {
    async fn exec(self, db: ServerDb, _db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        db.reset_tokens().await?;

        Ok(ClusterActionResult::None)
    }
}
