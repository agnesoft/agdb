use crate::next_user_name;
use crate::TestCluster;
use crate::TestServer;
use crate::ADMIN;

#[tokio::test]
async fn change_password() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let status = server
        .api
        .admin_user_change_password(owner, "password123")
        .await?;
    assert_eq!(status, 201);
    let status = server.api.user_login(owner, "password123").await?;
    assert_eq!(status, 200);
    Ok(())
}

#[tokio::test]
async fn password_too_short() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let status = server
        .api
        .admin_user_change_password(owner, "pswd")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 461);
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_user_change_password("owner", "password123")
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
        .admin_user_change_password(owner, "password123")
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
        .admin_user_change_password("owner", "password123")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn cluster_change_password() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let client = cluster.apis.get_mut(1).unwrap();
    let user = &next_user_name();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(user, user).await?;
    client
        .admin_user_change_password(user, "password123")
        .await?;
    client.user_login(user, "password123").await?;
    Ok(())
}
