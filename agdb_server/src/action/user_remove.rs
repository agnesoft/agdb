use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::AgdbDeSerialize;
use agdb::UserValue;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, UserValue, AgdbDeSerialize)]
pub(crate) struct UserRemove {
    pub(crate) user: String,
}

impl Action for UserRemove {
    async fn exec(self, db: ServerDb, db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        let dbs = db.remove_user(&self.user).await?;
        db_pool.remove_user_dbs(&self.user, &dbs).await?;
        Ok(ClusterActionResult::None)
    }
}
