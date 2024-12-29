use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::UserValue;
use agdb_api::DbResource;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, UserValue)]
pub(crate) struct DbClear {
    pub(crate) owner: String,
    pub(crate) db: String,
    pub(crate) resource: DbResource,
}

impl Action for DbClear {
    async fn exec(self, db: ServerDb, db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        let user = db.user_id(&self.owner).await?;
        let mut database = db.user_db(user, &self.owner, &self.db).await?;
        db_pool
            .clear_db(&self.owner, &self.db, &mut database, self.resource)
            .await?;
        db.save_db(&database).await?;
        Ok(ClusterActionResult::None)
    }
}
