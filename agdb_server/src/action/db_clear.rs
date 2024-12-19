use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::Config;
use crate::server_error::ServerResult;
use crate::utilities::db_name;
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
    async fn exec(self, db: ServerDb, db_pool: DbPool, config: &Config) -> ServerResult {
        let owner = db.user_id(&self.owner).await?;
        let name = db_name(&self.owner, &self.db);
        let mut database = db.user_db(owner, &name).await?;
        db_pool
            .clear_db(&self.owner, &self.db, &mut database, config, self.resource)
            .await?;
        db.save_db(&database).await
    }
}
