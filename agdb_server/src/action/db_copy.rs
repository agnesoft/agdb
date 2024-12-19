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
pub(crate) struct DbCopy {
    pub(crate) owner: String,
    pub(crate) db: String,
    pub(crate) new_owner: String,
    pub(crate) new_db: String,
    pub(crate) db_type: DbType,
}

impl Action for DbCopy {
    async fn exec(self, db: ServerDb, db_pool: DbPool, config: &Config) -> ServerResult {
        let name = db_name(&self.owner, &self.db);
        let target_name = db_name(&self.new_owner, &self.new_db);
        let new_owner_id = db.user_id(&self.new_owner).await?;

        db_pool
            .copy_db(&name, &self.new_owner, &self.new_db, &target_name, config)
            .await?;

        db.insert_db(
            new_owner_id,
            Database {
                db_id: None,
                name: target_name,
                db_type: self.db_type,
                backup: 0,
            },
        )
        .await?;

        Ok(())
    }
}
