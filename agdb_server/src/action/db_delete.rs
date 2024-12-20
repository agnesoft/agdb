use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::Config;
use crate::server_error::ServerResult;
use crate::utilities::db_name;
use agdb::UserValue;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, UserValue)]
pub(crate) struct DbDelete {
    pub(crate) owner: String,
    pub(crate) db: String,
}

impl Action for DbDelete {
    async fn exec(self, db: ServerDb, db_pool: DbPool, config: &Config) -> ServerResult {
        let name = db_name(&self.owner, &self.db);
        let user_id = db.user_id(&self.owner).await?;

        db.remove_db(user_id, &name).await?;
        db_pool
            .delete_db(&self.owner, &self.db, &name, config)
            .await?;

        Ok(())
    }
}
