use crate::create_cluster;
use crate::wait_for_leader;
use crate::TestCluster;
use crate::TestServer;
use crate::ADMIN;
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;
use assert_cmd::cargo::CommandCargoExt;
use std::process::Command;

#[tokio::test]
async fn rebalance() -> anyhow::Result<()> {
    let mut servers = create_cluster(3).await?;
    let mut leader = AgdbApi::new(ReqwestClient::new(), &servers[0].address);
    leader.user_login(ADMIN, ADMIN).await?;
    leader.admin_shutdown().await?;
    assert!(servers[0].process.wait()?.success());

    let mut statuses = Vec::with_capacity(servers.len());

    for server in &servers[1..] {
        let status = wait_for_leader(&AgdbApi::new(ReqwestClient::new(), &server.address)).await?;
        statuses.push(status);
    }

    for status in &statuses {
        assert_eq!(statuses[0], *status);
    }

    let dir = &servers[0].dir;
    servers[0].process = Command::cargo_bin("agdb_server")?
        .current_dir(dir)
        .spawn()?;

    statuses.clear();

    for server in &servers {
        let status = wait_for_leader(&AgdbApi::new(ReqwestClient::new(), &server.address)).await?;
        statuses.push(status);
    }

    for status in &statuses {
        assert_eq!(statuses[0], *status);
    }

    Ok(())
}

#[tokio::test]
async fn user() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;

    {
        let client = cluster.apis.get_mut(1).unwrap();
        client.cluster_login(ADMIN, ADMIN).await?;
        client.admin_user_add("user1", "password123").await?;
        client.user_login("user1", "password123").await?;
    }

    {
        let leader = cluster.apis.get_mut(0).unwrap();
        leader.user_login(ADMIN, ADMIN).await?;
        leader.admin_cluster_logout("user1").await?;
        leader.admin_user_remove("user1").await?;
    }

    {
        let client = cluster.apis.get_mut(1).unwrap();
        client.user_login(ADMIN, ADMIN).await?;
        client.user_status().await?;
        client.cluster_logout().await?;
        assert_eq!(client.user_status().await.unwrap_err().status, 401);
    }

    Ok(())
}

#[tokio::test]
async fn status() {
    let server = TestServer::new().await.unwrap();
    let (code, status) = server.api.cluster_status().await.unwrap();

    assert_eq!(code, 200);
    assert_eq!(status.len(), 0);
}
