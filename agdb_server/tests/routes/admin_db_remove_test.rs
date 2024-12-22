use crate::next_db_name;
use crate::next_user_name;
use crate::TestCluster;
use crate::TestServer;
use crate::ADMIN;
use agdb_api::DbType;
use std::path::Path;

#[tokio::test]
async fn remove() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    assert!(Path::new(&server.data_dir).join(owner).join(db).exists());
    let status = server.api.admin_db_remove(owner, db).await?;
    assert!(!server
        .api
        .admin_db_list()
        .await?
        .1
        .iter()
        .any(|d| d.name == format!("{}/{}", owner, db)));
    assert_eq!(status, 204);
    assert!(Path::new(&server.data_dir).join(owner).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let status = server
        .api
        .admin_db_remove(owner, db)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_db_remove(owner, db)
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
        .admin_db_remove(owner, db)
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
        .admin_db_remove("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn cluster_remove() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.cluster_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbType::Memory).await?;
    let admin_token = client.token.clone();
    client.user_login(owner, owner).await?;
    let user_token = client.token.clone();
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 1);
    client.token = admin_token;
    client.admin_db_remove(owner, db).await?;
    client.token = user_token;
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 0);
    Ok(())
}
