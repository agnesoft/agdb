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

const LEADER_TIMOUT: u128 = 10000;

async fn wait_for_leader(
    client: Arc<AgdbApi<ReqwestClient>>,
) -> anyhow::Result<Vec<ClusterStatus>> {
    let now = Instant::now();

    while now.elapsed().as_millis() < LEADER_TIMOUT {
        let status = client.cluster_status().await?;
        if status.1.iter().any(|s| s.leader) {
            return Ok(status.1);
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }

    Err(anyhow::anyhow!(
        "Leader not found within {LEADER_TIMOUT}seconds"
    ))
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
        config.insert("log_level", "INFO".into());
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

    let status = servers[1].1.user_status().await?.1;
    assert_eq!(status.name, ADMIN);
    assert!(status.admin);
    assert!(status.login);

    Ok(())
}

#[tokio::test]
async fn cluster_status() {
    let server = TestServer::new().await.unwrap();
    let (code, status) = server.api.cluster_status().await.unwrap();

    assert_eq!(code, 200);
    assert_eq!(status.len(), 0);
}
