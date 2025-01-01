use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::AgdbDeSerialize;
use agdb_api::Queries;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, AgdbDeSerialize)]
pub(crate) struct DbExec {
    pub(crate) user: String,
    pub(crate) owner: String,
    pub(crate) db: String,
    pub(crate) queries: Queries,
}

impl Action for DbExec {
    async fn exec(self, _db: ServerDb, db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        Ok(ClusterActionResult::QueryResults(
            db_pool
                .exec_mut(&self.owner, &self.db, &self.user, self.queries)
                .await?,
        ))
    }
}
