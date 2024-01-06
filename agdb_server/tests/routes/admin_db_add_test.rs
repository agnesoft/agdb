use agdb_api::DbType;

use crate::TestServer;
use crate::ADMIN;
use std::path::Path;

#[tokio::test]
async fn add() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let db = &server.next_db_name();
    let status = server.api.admin_db_add(owner, db, DbType::File).await?;
    assert_eq!(status.0, 201);
    assert!(Path::new(&server.data_dir).join(owner).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn add_same_name_with_previous_backup() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let db = &server.next_db_name();
    let status = server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    assert_eq!(status.0, 201);
    server.api.admin_db_backup(owner, db).await?;
    server.api.admin_db_delete(owner, db).await?;
    let status = server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    assert_eq!(status.0, 201);
    server.api.user_login(owner, owner).await?;
    let list = server.api.db_list().await?.1;
    assert_eq!(list[0].backup, 0);
    Ok(())
}

#[tokio::test]
async fn db_already_exists() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let db = &server.next_db_name();
    let status = server.api.admin_db_add(owner, db, DbType::File).await?;
    assert_eq!(status.0, 201);
    let status = server
        .api
        .admin_db_add(owner, db, DbType::File)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 465);
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_db_add("user", "db", DbType::Mapped)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let owner = &server.next_user_name();
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let db = &server.next_db_name();
    let status = server
        .api
        .admin_db_add(owner, db, DbType::Mapped)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_db_add("not_found", "not_found", DbType::Memory)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}
