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
pub(crate) struct DbUserAdd {
    pub(crate) owner: String,
    pub(crate) db: String,
    pub(crate) user: String,
    pub(crate) db_role: DbUserRole,
}

impl Action for DbUserAdd {
    async fn exec(
        self,
        db: ServerDb,
        _db_pool: DbPool,
        _config: &Config,
    ) -> ServerResult<ClusterActionResult> {
        let name = db_name(&self.owner, &self.db);
        let owner_id = db.user_id(&self.owner).await?;
        let db_id = db.user_db_id(owner_id, &name).await?;
        let user_id = db.user_id(&self.user).await?;
        db.insert_db_user(db_id, user_id, self.db_role).await?;

        Ok(ClusterActionResult::None)
    }
}
