use crate::next_db_name;
use crate::next_user_name;
use crate::TestCluster;
use crate::TestServer;
use crate::ADMIN;
use agdb_api::DbType;
use std::path::Path;

#[tokio::test]
async fn add() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let status = server.api.admin_db_add(owner, db, DbType::File).await?;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir).join(owner).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn add_same_name_with_previous_backup() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let status = server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    assert_eq!(status, 201);
    server.api.admin_db_backup(owner, db).await?;
    server.api.admin_db_delete(owner, db).await?;
    let status = server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    assert_eq!(status, 201);
    server.api.user_login(owner, owner).await?;
    let list = server.api.db_list().await?.1;
    assert_eq!(list[0].backup, 0);
    Ok(())
}

#[tokio::test]
async fn db_already_exists() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let status = server.api.admin_db_add(owner, db, DbType::File).await?;
    assert_eq!(status, 201);
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
        .admin_db_add("owner", "db", DbType::Mapped)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
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
        .admin_db_add("owner", "db", DbType::Memory)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn cluster_db_add() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.cluster_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbType::Memory).await?;
    let db_list = client.admin_db_list().await?.1;
    let server_db = db_list
        .iter()
        .find(|d| d.name == format!("{owner}/{db}"))
        .unwrap();
    assert_eq!(server_db.name, format!("{owner}/{db}"));
    assert_eq!(server_db.db_type, DbType::Memory);
    Ok(())
}
