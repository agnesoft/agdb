use crate::TestServer;
use crate::ADMIN;
use agdb_api::DbType;
use std::path::Path;

#[tokio::test]
async fn rename() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    let db2 = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    let status = server.api.admin_db_rename(owner, db, owner, db2).await?;
    assert_eq!(status, 201);
    assert!(!Path::new(&server.data_dir).join(owner).join(db).exists());
    assert!(Path::new(&server.data_dir).join(owner).join(db2).exists());
    Ok(())
}

#[tokio::test]
async fn rename_with_backup() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    let db2 = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    server.api.admin_db_backup(owner, db).await?;
    let status = server.api.admin_db_rename(owner, db, owner, db2).await?;
    assert_eq!(status, 201);
    assert!(!Path::new(&server.data_dir).join(owner).join(db).exists());
    assert!(!Path::new(&server.data_dir)
        .join(owner)
        .join("backups")
        .join(format!("{}.bak", db))
        .exists());
    assert!(Path::new(&server.data_dir).join(owner).join(db2).exists());
    assert!(Path::new(&server.data_dir)
        .join(owner)
        .join("backups")
        .join(format!("{}.bak", db2))
        .exists());
    Ok(())
}

#[tokio::test]
async fn transfer() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let owner2 = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(owner2, owner2).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    let status = server.api.admin_db_rename(owner, db, owner2, db).await?;
    assert_eq!(status, 201);
    assert!(!Path::new(&server.data_dir).join(owner).join(db).exists());
    assert!(Path::new(&server.data_dir).join(owner2).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    let status = server
        .api
        .admin_db_rename(owner, db, "not_found", db)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn invalid() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    let status = server
        .api
        .admin_db_rename(owner, db, owner, "a\0a")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 467);
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_db_rename(owner, "db", owner, "dbx")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn target_self() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    let status = server.api.admin_db_rename(owner, db, owner, db).await?;
    assert_eq!(status, 201);
    Ok(())
}

#[tokio::test]
async fn target_exists() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    let db2 = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    server.api.admin_db_add(owner, db2, DbType::Mapped).await?;
    let status = server
        .api
        .admin_db_rename(owner, db, owner, db2)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 465);
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .admin_db_rename(owner, "db", owner, "dbx")
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
        .admin_db_rename("owner", "db", "owner", "dbx")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}
