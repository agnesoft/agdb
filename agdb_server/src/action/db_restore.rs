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
pub(crate) struct DbRestore {
    pub(crate) owner: String,
    pub(crate) db: String,
}

impl Action for DbRestore {
    async fn exec(self, db: ServerDb, db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        let owner = db.user_id(&self.owner).await?;
        let name = db_name(&self.owner, &self.db);
        let mut database = db.user_db(owner, &name).await?;

        if let Some(backup) = db_pool
            .restore_db(&self.owner, &self.db, &name, database.db_type)
            .await?
        {
            database.backup = backup;
            db.save_db(&database).await?;
        }

        Ok(ClusterActionResult::None)
    }
}
