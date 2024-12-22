use crate::next_user_name;
use crate::TestCluster;
use crate::TestServer;
use crate::ADMIN;

#[tokio::test]
async fn login() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let status = server.api.user_login(ADMIN, ADMIN).await?;
    assert_eq!(status, 200);
    server.api.admin_user_add(owner, owner).await?;
    let status = server.api.user_login(owner, owner).await?;
    assert_eq!(status, 200);
    assert!(!server.api.token.clone().unwrap().is_empty());
    Ok(())
}

#[tokio::test]
async fn repeated_login() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let status = server.api.user_login(ADMIN, ADMIN).await?;
    assert_eq!(status, 200);
    server.api.admin_user_add(owner, owner).await?;
    let status = server.api.user_login(owner, owner).await?;
    assert_eq!(status, 200);
    let token = server.api.token.clone().unwrap();
    let status = server.api.user_login(owner, owner).await?;
    assert_eq!(status, 200);
    assert_eq!(server.api.token.clone().unwrap(), token);
    Ok(())
}

#[tokio::test]
async fn invalid_credentials() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let status = server.api.user_login(ADMIN, ADMIN).await?;
    assert_eq!(status, 200);
    server.api.admin_user_add(owner, owner).await?;
    let status = server
        .api
        .user_login(owner, "bad_password")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let status = server
        .api
        .user_login("owner", "bad_password")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn cluster_login() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;

    let token = {
        let client = cluster.apis.get_mut(1).unwrap();
        client.cluster_login(ADMIN, ADMIN).await?;
        client.token.clone()
    };

    let leader = cluster.apis.get_mut(0).unwrap();
    leader.token = token;
    leader.user_status().await?;

    Ok(())
}
