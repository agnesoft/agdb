use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_db::Database;
use crate::server_error::ServerResult;
use agdb::AgdbDeSerialize;
use agdb::UserValue;
use agdb_api::DbType;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, UserValue, AgdbDeSerialize)]
pub(crate) struct DbCopy {
    pub(crate) owner: String,
    pub(crate) db: String,
    pub(crate) new_owner: String,
    pub(crate) new_db: String,
    pub(crate) db_type: DbType,
}

impl Action for DbCopy {
    async fn exec(self, db: ServerDb, db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        let new_owner_id = db.user_id(&self.new_owner).await?;
        db_pool
            .copy_db(&self.owner, &self.db, &self.new_owner, &self.new_db)
            .await?;
        db.insert_db(
            new_owner_id,
            Database {
                db_id: None,
                db: self.new_db,
                owner: self.new_owner,
                db_type: self.db_type,
                backup: 0,
            },
        )
        .await?;
        Ok(ClusterActionResult::None)
    }
}
