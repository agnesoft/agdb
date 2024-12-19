use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::Config;
use crate::server_db::Database;
use crate::server_error::ServerResult;
use crate::utilities::db_name;
use agdb::UserValue;
use agdb_api::DbType;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, UserValue)]
pub(crate) struct DbAdd {
    pub(crate) owner: String,
    pub(crate) db: String,
    pub(crate) db_type: DbType,
}

impl Action for DbAdd {
    async fn exec(self, db: ServerDb, db_pool: DbPool, config: &Config) -> ServerResult {
        let name = db_name(&self.owner, &self.db);

        let backup = db_pool
            .add_db(&self.owner, &self.db, &name, self.db_type, config)
            .await?;

        let owner = db.user_id(&self.owner).await?;

        db.insert_db(
            owner,
            Database {
                db_id: None,
                name,
                db_type: self.db_type,
                backup,
            },
        )
        .await?;

        Ok(())
    }
}
