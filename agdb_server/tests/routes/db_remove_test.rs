use crate::TestServer;
use crate::ADMIN;
use agdb_api::DbType;
use agdb_api::DbUserRole;
use std::path::Path;

#[tokio::test]
async fn remove() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    assert!(Path::new(&server.data_dir).join(owner).join(db).exists());
    let status = server.api.db_remove(owner, db).await?;
    assert_eq!(status, 204);
    assert!(Path::new(&server.data_dir).join(owner).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn remove_with_backup() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    let db_path = Path::new(&server.data_dir).join(owner).join(db);
    let db_backup_path = Path::new(&server.data_dir)
        .join(owner)
        .join("backups")
        .join(format!("{}.bak", db));
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    server.api.db_backup(owner, db).await?;
    assert!(db_path.exists());
    assert!(db_backup_path.exists());
    let status = server.api.db_remove(owner, db).await?;
    assert_eq!(status, 204);
    assert!(db_path.exists());
    assert!(db_backup_path.exists());
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server.api.db_remove(owner, "db").await.unwrap_err().status;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn non_owner() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let user = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Admin)
        .await?;
    server.api.user_login(user, user).await?;
    let status = server.api.db_remove(owner, db).await.unwrap_err().status;
    assert_eq!(status, 403);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .db_remove("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}
