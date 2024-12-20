use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::action::Config;
use crate::server_error::ServerResult;
use crate::utilities::db_name;
use agdb::UserValue;
use agdb_api::DbType;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, UserValue)]
pub(crate) struct DbConvert {
    pub(crate) owner: String,
    pub(crate) db: String,
    pub(crate) db_type: DbType,
}

impl Action for DbConvert {
    async fn exec(
        self,
        db: ServerDb,
        db_pool: DbPool,
        config: &Config,
    ) -> ServerResult<ClusterActionResult> {
        let owner = db.user_id(&self.owner).await?;
        let name = db_name(&self.owner, &self.db);
        let mut database = db.user_db(owner, &name).await?;
        db_pool
            .convert_db(
                &self.owner,
                &self.db,
                &name,
                database.db_type,
                self.db_type,
                config,
            )
            .await?;
        database.db_type = self.db_type;
        db.save_db(&database).await?;
        Ok(ClusterActionResult::None)
    }
}
