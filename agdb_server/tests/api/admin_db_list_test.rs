use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb_api::DbKind;
use agdb_api::DbUserRole;

async fn db_list() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner1 = &next_user_name();
    let owner2 = &next_user_name();
    let db1 = &next_db_name();
    let db2 = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner1, owner1).await?;
    server.api.admin_user_add(owner2, owner2).await?;
    server.api.admin_db_add(owner1, db1, DbKind::Memory).await?;
    server.api.admin_db_add(owner2, db2, DbKind::Memory).await?;
    let (status, list) = server.api.admin_db_list().await?;
    assert_eq!(status, 200);
    let db1_entry = list
        .iter()
        .find(|d| d.db == *db1)
        .expect("the database is listed");
    assert_eq!(db1_entry.owner, *owner1);
    assert_eq!(db1_entry.db_type, DbKind::Memory);
    assert_eq!(db1_entry.role, DbUserRole::Admin);
    assert_ne!(db1_entry.created, 0);
    let db2_entry = list
        .iter()
        .find(|d| d.db == *db2)
        .expect("the database is listed");
    assert_eq!(db2_entry.owner, *owner2);
    assert_eq!(db2_entry.db_type, DbKind::Memory);
    assert_eq!(db2_entry.role, DbUserRole::Admin);
    assert_ne!(db2_entry.created, 0);
    Ok(())
}

async fn with_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    server.api.admin_db_backup(owner, db).await?;
    let (status, list) = server.api.admin_db_list().await?;
    assert_eq!(status, 200);
    let db = list
        .iter()
        .find(|d| d.db == *db && d.owner == *owner)
        .unwrap();
    assert_ne!(db.backup, 0);
    Ok(())
}

async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server.api.admin_db_list().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

async fn no_admin_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server.api.admin_db_list().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![db_list, with_backup, non_admin, no_admin_token,];
