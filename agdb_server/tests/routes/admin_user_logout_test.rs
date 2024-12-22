use crate::next_user_name;
use crate::TestCluster;
use crate::TestServer;
use crate::ADMIN;

#[tokio::test]
async fn logout() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let token = server.api.token.clone();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_logout(user).await?;
    server.api.token = token;
    let status = server.api.db_list().await.unwrap_err().status;
    assert_eq!(status, 401);

    Ok(())
}

#[tokio::test]
async fn unknown_user() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_user_logout("unknown_user")
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
    let status = server.api.admin_user_logout(user).await.unwrap_err().status;
    assert_eq!(status, 401);

    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_user_logout("user")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);

    Ok(())
}

#[tokio::test]
async fn cluster_user_logout() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let user = &next_user_name();

    {
        let client = cluster.apis.get_mut(1).unwrap();
        client.cluster_login(ADMIN, ADMIN).await?;
        client.admin_user_add(user, user).await?;
        client.cluster_login(user, user).await?;
    }

    cluster.apis[1].user_status().await?;
    let leader = cluster.apis.get_mut(0).unwrap();
    leader.user_login(ADMIN, ADMIN).await?;
    leader.admin_cluster_logout(user).await?;

    assert_eq!(cluster.apis[1].user_status().await.unwrap_err().status, 401);

    Ok(())
}
