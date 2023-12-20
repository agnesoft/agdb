use std::path::Path;

use crate::AddUser;
use crate::DbWithRole;
use crate::TestServer;
use crate::DB_LIST_URI;
use crate::DB_RENAME_URI;
use crate::DB_USER_ADD_URI;
use crate::NO_TOKEN;
use serde::Serialize;

#[derive(Serialize)]
struct DbRename {
    db: String,
    new_name: String,
}

#[tokio::test]
async fn rename() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
}
