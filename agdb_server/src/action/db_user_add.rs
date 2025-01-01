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
pub(crate) struct DbUserAdd {
    pub(crate) owner: String,
    pub(crate) db: String,
    pub(crate) user: String,
    pub(crate) db_role: DbUserRole,
}

impl Action for DbUserAdd {
    async fn exec(self, db: ServerDb, _db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        let owner_id = db.user_id(&self.owner).await?;
        let db_id = db.user_db_id(owner_id, &self.owner, &self.db).await?;
        let user_id = db.user_id(&self.user).await?;
        db.insert_db_user(db_id, user_id, self.db_role).await?;

        Ok(ClusterActionResult::None)
    }
}
