use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::action::Config;
use crate::server_error::ServerResult;
use crate::utilities::db_name;
use agdb::UserValue;
use agdb_api::DbUserRole;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, UserValue)]
pub(crate) struct DbRename {
    pub(crate) owner: String,
    pub(crate) db: String,
    pub(crate) new_owner: String,
    pub(crate) new_db: String,
}

impl Action for DbRename {
    async fn exec(
        self,
        db: ServerDb,
        db_pool: DbPool,
        config: &Config,
    ) -> ServerResult<ClusterActionResult> {
        let owner_id = db.user_id(&self.owner).await?;
        let name = db_name(&self.owner, &self.db);
        let new_name = db_name(&self.new_owner, &self.new_db);
        let mut database = db.user_db(owner_id, &name).await?;
        db_pool
            .rename_db(
                &self.owner,
                &self.db,
                &name,
                &self.new_owner,
                &self.new_db,
                &new_name,
                config,
            )
            .await?;
        database.name = new_name;
        db.save_db(&database).await?;

        if self.owner != self.new_owner {
            let new_owner_id = db.user_id(&self.new_owner).await?;
            db.insert_db_user(
                database.db_id.expect("database should have db_id"),
                new_owner_id,
                DbUserRole::Admin,
            )
            .await?;
        }

        Ok(ClusterActionResult::None)
    }
}
