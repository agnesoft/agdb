use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb_api::DbKind;

async fn add() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let status = server.api.admin_db_add(owner, db, DbKind::File).await?;
    assert_eq!(status, 201);
    assert!(
        server
            .api
            .admin_db_list()
            .await?
            .1
            .iter()
            .any(|d| d.owner == *owner && d.db == *db)
    );
    Ok(())
}

async fn add_same_name_with_previous_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let status = server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    assert_eq!(status, 201);
    server.api.admin_db_backup(owner, db).await?;
    server.api.admin_db_delete(owner, db).await?;
    let status = server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    assert_eq!(status, 201);
    server.api.user_login(owner, owner).await?;
    let list = server.api.db_list().await?.1;
    assert_eq!(list[0].backup, 0);
    Ok(())
}

async fn db_already_exists() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let status = server.api.admin_db_add(owner, db, DbKind::File).await?;
    assert_eq!(status, 201);
    let status = server
        .api
        .admin_db_add(owner, db, DbKind::File)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 465);
    Ok(())
}

async fn user_not_found() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_db_add("owner", "db", DbKind::Mapped)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .admin_db_add(owner, db, DbKind::Mapped)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_db_add("owner", "db", DbKind::Memory)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![
    add,
    add_same_name_with_previous_backup,
    db_already_exists,
    user_not_found,
    non_admin,
    no_token,
];
