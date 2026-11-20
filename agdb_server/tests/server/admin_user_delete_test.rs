use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb_api::DbKind;
use agdb_api::DbUserRole;
use std::path::Path;

async fn delete_with_other() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbKind::File).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    server.api.admin_user_delete(owner).await?;
    assert!(
        !server
            .api
            .admin_user_list()
            .await?
            .1
            .iter()
            .any(|u| u.username == *owner)
    );
    assert!(!Path::new(&server.data_dir).join(owner).exists());
    server.api.user_login(user, user).await?;
    assert!(server.api.db_list().await?.1.is_empty());
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![delete_with_other,];
