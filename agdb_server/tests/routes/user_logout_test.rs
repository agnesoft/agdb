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
async fn cluster_user_logout() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let user = &next_user_name();

    let token = {
        let client = cluster.apis.get_mut(1).unwrap();
        client.user_login(ADMIN, ADMIN).await?;
        client.admin_user_add(user, user).await?;
        client.cluster_login(user, user).await?;
        client.token.clone()
    };

    {
        let leader = cluster.apis.get_mut(0).unwrap();
        leader.token = token;
        leader.user_status().await?;
    }

    {
        let client = cluster.apis.get_mut(1).unwrap();
        client.cluster_logout().await?;
    }

    assert_eq!(cluster.apis[0].user_status().await.unwrap_err().status, 401);
    assert_eq!(cluster.apis[1].user_status().await.unwrap_err().status, 401);

    Ok(())
}
