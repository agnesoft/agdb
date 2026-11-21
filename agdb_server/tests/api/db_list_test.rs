use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb_api::DbKind;
use agdb_api::DbUserRole;

async fn list() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db1 = &next_db_name();
    let db2 = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db1, DbKind::Memory).await?;
    server.api.admin_db_add(user, db2, DbKind::Memory).await?;
    server
        .api
        .admin_db_user_add(owner, db1, user, DbUserRole::Read)
        .await?;
    server.api.user_login(user, user).await?;
    let (status, list) = server.api.db_list().await?;
    assert_eq!(status, 200);
    assert_eq!(list.len(), 2, "{list:?}");
    let db1_entry = list
        .iter()
        .find(|d| d.db == *db1)
        .expect("the database is listed");
    assert_eq!(db1_entry.owner, *owner);
    assert_eq!(db1_entry.db_type, DbKind::Memory);
    assert_eq!(db1_entry.role, DbUserRole::Read);
    let db2_entry = list
        .iter()
        .find(|d| d.db == *db2)
        .expect("the database is listed");
    assert_eq!(db2_entry.owner, *user);
    assert_eq!(db2_entry.db_type, DbKind::Memory);
    assert_eq!(db2_entry.role, DbUserRole::Admin);
    Ok(())
}

async fn with_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    server.api.db_backup(owner, db).await?;
    let (status, list) = server.api.db_list().await?;
    assert_eq!(status, 200);
    let db = list
        .iter()
        .find(|d| d.db == *db && d.owner == *owner)
        .unwrap();
    assert_ne!(db.backup, 0);
    Ok(())
}

async fn list_empty() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let (status, list) = server.api.db_list().await?;
    assert_eq!(status, 200);
    assert!(list.is_empty());
    Ok(())
}

async fn list_no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server.api.db_list().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![list, with_backup, list_empty, list_no_token,];
