use crate::TestServer;
use crate::TestServerImpl;
use crate::ADMIN;
use crate::HOST;
use crate::SERVER_DATA_DIR;
use agdb_api::AgdbApi;
use agdb_api::ClusterStatus;
use agdb_api::ReqwestClient;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

async fn wait_for_leader(
    client: Arc<AgdbApi<ReqwestClient>>,
) -> anyhow::Result<Vec<ClusterStatus>> {
    let now = Instant::now();

    while now.elapsed().as_millis() < 10000 {
        let status = client.cluster_status().await?;
        if status.1.iter().any(|s| s.leader) {
            return Ok(status.1);
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }

    Err(anyhow::anyhow!("Leader not found within 10 seconds"))
}

async fn create_cluster(
    nodes: usize,
) -> anyhow::Result<Vec<(TestServerImpl, Arc<AgdbApi<ReqwestClient>>)>> {
    let mut configs = Vec::with_capacity(nodes);
    let mut cluster = Vec::with_capacity(nodes);
    let mut servers = Vec::with_capacity(nodes);
    let mut statuses = Vec::with_capacity(nodes);

    for _ in 0..nodes {
        let port = TestServerImpl::next_port();
        let mut config = HashMap::<&str, serde_yaml::Value>::new();
        config.insert("bind", format!("{HOST}:{port}").into());
        config.insert("address", format!("http://{HOST}:{port}").into());
        config.insert("admin", ADMIN.into());
        config.insert("basepath", "".into());
        config.insert("data_dir", SERVER_DATA_DIR.into());
        config.insert("cluster_token", "test".into());

        configs.push(config);
        cluster.push(format!("http://{HOST}:{port}"));
    }

    for config in &mut configs {
        config.insert("cluster", cluster.clone().into());
    }

    for config in configs {
        let server = TestServerImpl::with_config(config).await?;
        let client = Arc::new(AgdbApi::new(ReqwestClient::new(), &server.address));
        servers.push((server, client));
    }

    for has_leader in servers
        .iter()
        .map(|(_, c)| tokio::spawn(wait_for_leader(c.clone())))
    {
        statuses.push(has_leader.await??);
    }

    for status in &statuses[1..] {
        assert_eq!(statuses[0], *status);
    }

    Ok(servers)
}

#[tokio::test]
#[ignore]
async fn cluster_rebalance() -> anyhow::Result<()> {
    let mut servers = create_cluster(3).await?;

    let mut client = AgdbApi::new(ReqwestClient::new(), &servers[0].0.address);
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_shutdown().await?;
    assert!(servers[0].0.process.wait()?.success());

    let mut statuses = Vec::with_capacity(3);

    for has_leader in servers[1..]
        .iter()
        .map(|(_, c)| tokio::spawn(wait_for_leader(c.clone())))
    {
        statuses.push(has_leader.await??);
    }

    for status in &statuses[1..] {
        assert_eq!(statuses[0], *status);
    }

    Ok(())
}

#[tokio::test]
async fn cluster_status() {
    let server = TestServer::new().await.unwrap();
    let (code, status) = server.api.cluster_status().await.unwrap();

    assert_eq!(code, 200);
    assert_eq!(status.len(), 0);
}

#[tokio::test]
async fn heartbeat_no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let client = reqwest::Client::new();
    let status = client
        .post(server.full_url("/cluster/heartbeat?cluster_hash=test&term=1&leader=0"))
        .send()
        .await?
        .status();

    assert_eq!(status, 401);

    Ok(())
}

#[tokio::test]
async fn vote_no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let client = reqwest::Client::new();
    let status = client
        .get(server.full_url("/cluster/vote?cluster_hash=test&term=1&leader=0"))
        .send()
        .await?
        .status();

    assert_eq!(status, 401);

    Ok(())
}
