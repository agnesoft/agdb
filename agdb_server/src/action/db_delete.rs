use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::DbSerialize;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, DbSerialize)]
pub(crate) struct DbDelete {
    pub(crate) owner: String,
    pub(crate) db: String,
}

impl Action for DbDelete {
    async fn exec(self, db: ServerDb, db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        let user_id = db.user_id(&self.owner).await?;
        db.remove_db(user_id, &self.owner, &self.db).await?;
        db_pool.delete_db(&self.owner, &self.db).await?;
        Ok(ClusterActionResult::None)
    }
}
