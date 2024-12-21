use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use crate::utilities::db_name;
use agdb::UserValue;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, UserValue)]
pub(crate) struct DbBackup {
    pub(crate) owner: String,
    pub(crate) db: String,
}

impl Action for DbBackup {
    async fn exec(self, db: ServerDb, db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        let owner = db.user_id(&self.owner).await?;
        let name = db_name(&self.owner, &self.db);
        let mut database = db.user_db(owner, &name).await?;
        database.backup = db_pool
            .backup_db(&self.owner, &self.db, &name, database.db_type)
            .await?;
        db.save_db(&database).await?;
        Ok(ClusterActionResult::None)
    }
}
