use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb_api::DbKind;
use std::path::Path;

async fn remove() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    assert!(Path::new(&server.data_dir).join(owner).join(db).exists());
    let status = server.api.admin_db_remove(owner, db).await?;
    assert!(
        !server
            .api
            .admin_db_list()
            .await?
            .1
            .iter()
            .any(|d| d.db == *db && d.owner == *owner)
    );
    assert_eq!(status, 204);
    assert!(Path::new(&server.data_dir).join(owner).join(db).exists());
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![remove,];
