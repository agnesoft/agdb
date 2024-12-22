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
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .user_change_password(owner, "password123")
        .await?;
    assert_eq!(status, 201);
    let status = server.api.user_login(owner, "password123").await?;
    assert_eq!(status, 200);
    Ok(())
}

#[tokio::test]
async fn invalid_credentials() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .user_change_password("bad_password", "password123")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn password_too_short() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .user_change_password(owner, "pswd")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 461);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .user_change_password("pswd", "pswd")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn cluster_change_password() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let user = &next_user_name();
    {
        let client = cluster.apis.get_mut(1).unwrap();
        client.user_login(ADMIN, ADMIN).await?;
        client.admin_user_add(user, user).await?;
        client.cluster_login(user, user).await?;
        client.user_change_password(user, "password123").await?;
    }
    cluster.apis[0].user_login(user, "password123").await?;
    Ok(())
}
