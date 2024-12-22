use crate::next_db_name;
use crate::next_user_name;
use crate::TestCluster;
use crate::TestServer;
use crate::ADMIN;
use agdb_api::DbType;
use agdb_api::DbUserRole;
use std::path::Path;

#[tokio::test]
async fn remove() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    let status = server.api.admin_user_remove(user).await?;
    assert_eq!(status, 204);
    assert!(!server
        .api
        .admin_user_list()
        .await?
        .1
        .iter()
        .any(|u| u.name == *user));
    Ok(())
}

#[tokio::test]
async fn remove_with_other() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbType::File).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    server.api.admin_user_remove(owner).await?;
    assert!(!server
        .api
        .admin_user_list()
        .await?
        .1
        .iter()
        .any(|u| u.name == *owner));
    assert!(!Path::new(&server.data_dir).join(owner).exists());
    server.api.user_login(user, user).await?;
    assert!(server.api.db_list().await?.1.is_empty());
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_user_remove("not_found")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server.api.admin_user_remove(user).await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_user_remove("user")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn cluster_user_remove() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let client = cluster.apis.get_mut(1).unwrap();
    let user = &next_user_name();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(user, user).await?;
    let users = client.admin_user_list().await?.1;
    let added_user = users.iter().find(|u| u.name.as_str() == user);
    assert!(added_user.is_some());
    client.admin_user_remove(user).await?;
    let users = client.admin_user_list().await?.1;
    let added_user = users.iter().find(|u| u.name.as_str() == user);
    assert!(added_user.is_none());
    Ok(())
}
