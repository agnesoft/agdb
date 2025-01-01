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
pub(crate) struct DbAdd {
    pub(crate) owner: String,
    pub(crate) db: String,
    pub(crate) db_type: DbType,
}

impl Action for DbAdd {
    async fn exec(self, db: ServerDb, db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        let backup = db_pool.add_db(&self.owner, &self.db, self.db_type).await?;
        let owner = db.user_id(&self.owner).await?;

        db.insert_db(
            owner,
            Database {
                db_id: None,
                db: self.db,
                owner: self.owner,
                db_type: self.db_type,
                backup,
            },
        )
        .await?;

        Ok(ClusterActionResult::None)
    }
}
