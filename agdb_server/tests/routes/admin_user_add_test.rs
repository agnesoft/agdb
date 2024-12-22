use crate::next_user_name;
use crate::TestCluster;
use crate::TestServer;
use crate::ADMIN;

#[tokio::test]
async fn add() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server.api.admin_user_add(user, user).await?;
    assert_eq!(status, 201);
    Ok(())
}

#[tokio::test]
async fn add_existing() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    let status = server
        .api
        .admin_user_add(user, user)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 463);
    Ok(())
}

#[tokio::test]
async fn name_too_short() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_user_add("a", "password123")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 462);
    Ok(())
}

#[tokio::test]
async fn password_too_short() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_user_add("user123", "pswd")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 461);
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .admin_user_add(user, user)
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
        .admin_user_add("user", "password123")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn cluster_user_add() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let client = cluster.apis.get_mut(1).unwrap();
    let user = &next_user_name();
    client.cluster_login(ADMIN, ADMIN).await?;
    client.admin_user_add(user, user).await?;
    let users = client.admin_user_list().await?.1;
    let added_user = users.iter().find(|u| u.name.as_str() == user);
    assert!(added_user.is_some());
    Ok(())
}
