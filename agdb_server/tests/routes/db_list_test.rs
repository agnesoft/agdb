use crate::TestServer;
use crate::ADMIN;
use agdb_api::DbType;
use agdb_api::DbUserRole;
use agdb_api::ServerDatabase;

#[tokio::test]
async fn list() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let user = &server.next_user_name();
    let db1 = &server.next_db_name();
    let db2 = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db1, DbType::Memory).await?;
    server.api.admin_db_add(user, db2, DbType::Memory).await?;
    server
        .api
        .admin_db_user_add(owner, db1, user, DbUserRole::Read)
        .await?;
    server.api.user_login(user, user).await?;
    let (status, mut list) = server.api.db_list().await?;
    assert_eq!(status, 200);
    let mut expected = vec![
        ServerDatabase {
            name: format!("{}/{}", owner, db1),
            db_type: DbType::Memory,
            role: DbUserRole::Read,
            size: 2632,
            backup: 0,
        },
        ServerDatabase {
            name: format!("{}/{}", user, db2),
            db_type: DbType::Memory,
            role: DbUserRole::Admin,
            size: 2632,
            backup: 0,
        },
    ];
    list.sort();
    expected.sort();
    assert_eq!(list, expected);
    Ok(())
}

#[tokio::test]
async fn with_backup() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    server.api.db_backup(owner, db).await?;
    let (status, list) = server.api.db_list().await?;
    assert_eq!(status, 200);
    let db = list
        .iter()
        .find(|d| d.name == format!("{}/{}", owner, db))
        .unwrap();
    assert_ne!(db.backup, 0);
    Ok(())
}

#[tokio::test]
async fn list_empty() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let (status, list) = server.api.db_list().await?;
    assert_eq!(status, 200);
    assert!(list.is_empty());
    Ok(())
}

#[tokio::test]
async fn list_no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server.api.db_list().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}
