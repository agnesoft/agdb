use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::AgdbDeSerialize;
use agdb::UserValue;
use agdb_api::DbUserRole;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, UserValue, AgdbDeSerialize)]
pub(crate) struct DbRename {
    pub(crate) owner: String,
    pub(crate) db: String,
    pub(crate) new_owner: String,
    pub(crate) new_db: String,
}

impl Action for DbRename {
    async fn exec(self, db: ServerDb, db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        let owner_id = db.user_id(&self.owner).await?;
        let mut database = db.user_db(owner_id, &self.owner, &self.db).await?;
        db_pool
            .rename_db(&self.owner, &self.db, &self.new_owner, &self.new_db)
            .await?;
        database.owner = self.new_owner.clone();
        database.db = self.new_db.to_string();
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
