use crate::ADMIN;
use crate::TestServer;
use crate::next_db_name;
use crate::next_user_name;
use agdb_api::DbKind;
use agdb_api::DbUserRole;
use std::path::Path;

#[tokio::test]
async fn rename() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let status = server.api.db_rename(owner, db, db2).await?;
    assert_eq!(status, 201);
    assert!(!Path::new(&server.data_dir).join(owner).join(db).exists());
    assert!(Path::new(&server.data_dir).join(owner).join(db2).exists());
    Ok(())
}

#[tokio::test]
async fn rename_with_backup() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    server.api.db_backup(owner, db).await?;
    let status = server.api.db_rename(owner, db, db2).await?;
    assert_eq!(status, 201);
    assert!(!Path::new(&server.data_dir).join(owner).join(db).exists());
    assert!(
        !Path::new(&server.data_dir)
            .join(owner)
            .join("backups")
            .join(format!("{db}.bak"))
            .exists()
    );
    assert!(Path::new(&server.data_dir).join(owner).join(db2).exists());
    assert!(
        Path::new(&server.data_dir)
            .join(owner)
            .join("backups")
            .join(format!("{db2}.bak"))
            .exists()
    );
    Ok(())
}

#[tokio::test]
async fn non_owner() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Admin)
        .await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .db_rename(owner, db, "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

#[tokio::test]
async fn invalid() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let status = server
        .api
        .db_rename(owner, db, "a\0a")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 467);
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .db_rename(owner, "db", "not_found")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn target_self() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let status = server.api.db_rename(owner, db, db).await?;
    assert_eq!(status, 201);
    Ok(())
}

#[tokio::test]
async fn target_exists() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    server.api.admin_db_add(owner, db2, DbKind::Mapped).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .db_rename(owner, db, db2)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 465);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .db_rename("owner", "db", "dbx")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}
