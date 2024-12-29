use crate::next_db_name;
use crate::next_user_name;
use crate::TestServer;
use crate::ADMIN;
use agdb_api::DbType;
use agdb_api::DbUserRole;
use agdb_api::ServerDatabase;

#[tokio::test]
async fn db_list() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner1 = &next_user_name();
    let owner2 = &next_user_name();
    let db1 = &next_db_name();
    let db2 = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner1, owner1).await?;
    server.api.admin_user_add(owner2, owner2).await?;
    server.api.admin_db_add(owner1, db1, DbType::Memory).await?;
    server.api.admin_db_add(owner2, db2, DbType::Memory).await?;
    let (status, list) = server.api.admin_db_list().await?;
    assert_eq!(status, 200);
    assert!(list.contains(&ServerDatabase {
        db: db1.to_string(),
        owner: owner1.to_string(),
        db_type: DbType::Memory,
        role: DbUserRole::Admin,
        size: 2656,
        backup: 0,
    }));
    assert!(list.contains(&ServerDatabase {
        db: db2.to_string(),
        owner: owner2.to_string(),
        db_type: DbType::Memory,
        role: DbUserRole::Admin,
        size: 2656,
        backup: 0,
    }));
    Ok(())
}

#[tokio::test]
async fn with_backup() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
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

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server.api.admin_db_list().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn no_admin_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server.api.admin_db_list().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}
