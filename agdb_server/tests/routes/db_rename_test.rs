use crate::next_db_name;
use crate::next_user_name;
use crate::TestServer;
use crate::ADMIN;
use agdb_api::DbType;
use agdb_api::DbUserRole;
use agdb_api::ServerDatabase;
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
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let status = server.api.db_rename(owner, db, owner, db2).await?;
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
    server.api.db_add(owner, db, DbType::Mapped).await?;
    server.api.db_backup(owner, db).await?;
    let status = server.api.db_rename(owner, db, owner, db2).await?;
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
    let owner = &next_user_name();
    let owner2 = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(owner2, owner2).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let status = server.api.db_rename(owner, db, owner2, db).await?;
    assert_eq!(status, 201);
    server.api.user_login(owner2, owner2).await?;
    let list = server.api.db_list().await?.1;
    let expected = vec![ServerDatabase {
        db: db.to_string(),
        owner: owner2.to_string(),
        db_type: DbType::Mapped,
        role: DbUserRole::Admin,
        size: 2656,
        backup: 0,
    }];
    assert_eq!(list, expected);
    assert!(!Path::new(&server.data_dir).join(owner).join(db).exists());
    assert!(Path::new(&server.data_dir).join(owner2).join(db).exists());
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
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Admin)
        .await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .db_rename(owner, db, owner, "db")
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
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let status = server
        .api
        .db_rename(owner, db, owner, "a\0a")
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
        .db_rename(owner, "db", owner, "not_found")
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
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let status = server.api.db_rename(owner, db, owner, db).await?;
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
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    server.api.admin_db_add(owner, db2, DbType::Mapped).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .db_rename(owner, db, owner, db2)
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
        .db_rename("owner", "db", "owner", "dbx")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}
