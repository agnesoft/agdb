use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::AgdbDeSerialize;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, AgdbDeSerialize)]
pub(crate) struct DbBackup {
    pub(crate) owner: String,
    pub(crate) db: String,
}

impl Action for DbBackup {
    async fn exec(self, db: ServerDb, db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        let user = db.user_id(&self.owner).await?;
        let mut database = db.user_db(user, &self.owner, &self.db).await?;
        database.backup = db_pool
            .backup_db(&self.owner, &self.db, database.db_type)
            .await?;
        db.save_db(&database).await?;
        Ok(ClusterActionResult::None)
    }
}
