use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::DbSerialize;
use agdb_api::DbKind;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, DbSerialize)]
pub(crate) struct DbConvert {
    pub(crate) owner: String,
    pub(crate) db: String,
    pub(crate) db_type: DbKind,
}

impl Action for DbConvert {
    async fn exec(self, db: ServerDb, db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        let user = db.user_id(&self.owner).await?;
        let mut database = db.user_db(user, &self.owner, &self.db).await?;
        db_pool
            .convert_db(&self.owner, &self.db, database.db_type, self.db_type)
            .await?;
        database.db_type = self.db_type;
        db.save_db(&database).await?;
        Ok(ClusterActionResult::None)
    }
}
