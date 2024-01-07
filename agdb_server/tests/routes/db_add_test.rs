use crate::TestServer;
use crate::ADMIN;
use agdb_api::DbType;
use std::path::Path;

#[tokio::test]
async fn add() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server.api.db_add(owner, db, DbType::File).await?;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir).join(owner).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn add_same_name_with_previous_backup() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server.api.db_add(owner, db, DbType::Mapped).await?;
    assert_eq!(status, 201);
    server.api.db_backup(owner, db).await?;
    server.api.db_delete(owner, db).await?;
    let status = server.api.db_add(owner, db, DbType::Mapped).await?;
    assert_eq!(status, 201);
    let list = server.api.db_list().await?.1;
    assert_eq!(list[0].backup, 0);
    Ok(())
}

#[tokio::test]
async fn add_same_name_different_user() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let owner2 = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(owner2, owner2).await?;
    server.api.user_login(owner, owner).await?;
    let status = server.api.db_add(owner, db, DbType::File).await?;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir).join(owner).join(db).exists());
    server.api.user_login(owner2, owner2).await?;
    let status = server.api.db_add(owner2, db, DbType::File).await?;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir).join(owner2).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn db_already_exists() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server.api.db_add(owner, db, DbType::File).await?;
    assert_eq!(status, 201);
    let status = server
        .api
        .db_add(owner, db, DbType::File)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 465);
    Ok(())
}

#[tokio::test]
async fn db_user_mismatch() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .db_add("some_user", "db", DbType::Mapped)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

#[tokio::test]
async fn add_db_other_user() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let owner2 = &server.next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(owner2, owner2).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .db_add(owner2, "db", DbType::Mapped)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

#[tokio::test]
async fn db_type_invalid() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .db_add(owner, "a\0a", DbType::Mapped)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 467);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .db_add("owner", "db", DbType::Mapped)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}
