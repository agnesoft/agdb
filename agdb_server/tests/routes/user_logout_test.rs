use crate::next_user_name;
use crate::TestCluster;
use crate::TestServer;
use crate::ADMIN;

#[tokio::test]
async fn logout() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server.api.user_logout().await?;
    assert_eq!(status, 201);
    assert_eq!(server.api.token, None);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let status = server.api.user_logout().await.unwrap_err().status;
    assert_eq!(status, 401);

    Ok(())
}

#[tokio::test]
async fn cluster_logout() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;

    let token = {
        let leader = cluster.apis.get_mut(0).unwrap();
        leader.cluster_login(ADMIN, ADMIN).await?;
        leader.token.clone()
    };

    {
        let client = cluster.apis.get_mut(1).unwrap();
        client.token = token.clone();
        client.user_status().await?;
    }

    cluster.apis.get_mut(0).unwrap().cluster_logout().await?;
    assert_eq!(
        cluster
            .apis
            .get(1)
            .unwrap()
            .user_status()
            .await
            .unwrap_err()
            .status,
        401
    );

    Ok(())
}
