use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::AgdbDeSerialize;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, AgdbDeSerialize)]
pub(crate) struct DbRestore {
    pub(crate) owner: String,
    pub(crate) db: String,
}

impl Action for DbRestore {
    async fn exec(self, db: ServerDb, db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        let user = db.user_id(&self.owner).await?;
        let mut database = db.user_db(user, &self.owner, &self.db).await?;

        if let Some(backup) = db_pool
            .restore_db(&self.owner, &self.db, database.db_type)
            .await?
        {
            database.backup = backup;
            db.save_db(&database).await?;
        }

        Ok(ClusterActionResult::None)
    }
}
