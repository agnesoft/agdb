use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::DbSerialize;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, DbSerialize)]
pub(crate) struct DbAuditTrim {
    pub(crate) owner: String,
    pub(crate) db: String,
    pub(crate) first_day: u64,
}

impl Action for DbAuditTrim {
    async fn exec(self, _db: ServerDb, db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        db_pool
            .audit_trim(&self.owner, &self.db, self.first_day)
            .await?;
        Ok(ClusterActionResult::None)
    }
}
