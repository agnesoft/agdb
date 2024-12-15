use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterResponse;
use crate::action::Config;
use crate::server_error::ServerResult;
use agdb::UserValue;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, UserValue)]
pub(crate) struct ChangePassword {
    pub(crate) user: String,
    pub(crate) new_password: Vec<u8>,
    pub(crate) new_salt: Vec<u8>,
}

impl Action for ChangePassword {
    async fn exec(
        self,
        db: &mut ServerDb,
        _db_pool: &mut DbPool,
        _config: &Config,
    ) -> ServerResult<ClusterResponse> {
        let mut user = db.user(&self.user).await?;
        user.password = self.new_password;
        user.salt = self.new_salt;
        db.save_user(user).await?;

        Ok(ClusterResponse::None)
    }
}
