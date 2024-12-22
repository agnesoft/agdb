use crate::next_db_name;
use crate::next_user_name;
use crate::TestCluster;
use crate::TestServer;
use crate::ADMIN;
use agdb_api::DbType;
use agdb_api::DbUserRole;

#[tokio::test]
async fn remove() -> anyhow::Result<()> {
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
        .admin_db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    let list = server.api.admin_db_user_list(owner, db).await?.1;
    assert_eq!(list.len(), 2);
    let status = server.api.admin_db_user_remove(owner, db, user).await?;
    assert_eq!(status, 204);
    let list = server.api.admin_db_user_list(owner, db).await?.1;
    assert_eq!(list.len(), 1);
    Ok(())
}

#[tokio::test]
async fn remove_owner() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    let status = server
        .api
        .admin_db_user_remove(owner, db, owner)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let status = server
        .api
        .admin_db_user_remove(owner, "db", "user")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_db_user_remove("owner", "db", "user")
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
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .admin_db_user_remove(owner, "db", "user")
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
        .admin_db_user_remove("owner", "db", "user")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn cluster_db_user_add() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.cluster_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_user_add(user, user).await?;
    client.admin_db_add(owner, db, DbType::Memory).await?;
    client
        .admin_db_user_add(owner, db, user, DbUserRole::Read)
        .await?;
    let users = client.admin_db_user_list(owner, db).await?.1;
    assert_eq!(users.len(), 2);
    client.admin_db_user_remove(owner, db, user).await?;
    let users = client.admin_db_user_list(owner, db).await?.1;
    assert_eq!(users.len(), 1);
    Ok(())
}
