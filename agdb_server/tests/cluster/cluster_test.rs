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
use tokio::sync::RwLock;

const TEST_TIMEOUT: u128 = 10000;
const POLL_INTERVAL: u64 = 250;

type ClusterClient = Arc<RwLock<AgdbApi<ReqwestClient>>>;

struct ClusterServer {
    server: TestServerImpl,
    client: ClusterClient,
}

async fn wait_for_leader(client: ClusterClient) -> anyhow::Result<Vec<ClusterStatus>> {
    let now = Instant::now();

    while now.elapsed().as_millis() < TEST_TIMEOUT {
        let status = client.read().await.cluster_status().await?;

        if status.1.iter().any(|s| s.leader) {
            return Ok(status.1);
        }

        std::thread::sleep(std::time::Duration::from_millis(POLL_INTERVAL));
    }

    Err(anyhow::anyhow!(
        "Leader not found within {TEST_TIMEOUT}seconds"
    ))
}

async fn wait_for_user(client: ClusterClient, username: &str) -> anyhow::Result<()> {
    let now = Instant::now();

    while now.elapsed().as_millis() < TEST_TIMEOUT {
        if client
            .read()
            .await
            .admin_user_list()
            .await?
            .1
            .iter()
            .any(|u| u.name == username)
        {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(POLL_INTERVAL));
    }

    Err(anyhow::anyhow!(
        "User '{username}' not found within {TEST_TIMEOUT}seconds"
    ))
}

async fn create_cluster(nodes: usize) -> anyhow::Result<(ClusterServer, Vec<ClusterServer>)> {
    let mut configs = Vec::with_capacity(nodes);
    let mut cluster = Vec::with_capacity(nodes);
    let mut servers = Vec::with_capacity(nodes);
    let mut statuses = Vec::with_capacity(nodes);

    for _ in 0..nodes {
        let port = TestServerImpl::next_port();
        tracing::info!("NEW_PORT: {port}");
        println!("NEW_PORT: {port}");
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
        let client = Arc::new(RwLock::new(AgdbApi::new(
            ReqwestClient::new(),
            &server.address,
        )));
        servers.push(ClusterServer { server, client });
    }

    for has_leader in servers
        .iter()
        .map(|s| tokio::spawn(wait_for_leader(s.client.clone())))
    {
        statuses.push(has_leader.await??);
    }

    for status in &statuses[1..] {
        assert_eq!(statuses[0], *status);
    }

    let leader = statuses[0]
        .iter()
        .enumerate()
        .find_map(|(i, s)| if s.leader { Some(i) } else { None })
        .unwrap();

    Ok((servers.remove(leader), servers))
}

#[tokio::test]
async fn rebalance() -> anyhow::Result<()> {
    let (mut leader, servers) = create_cluster(3).await?;
    leader.client.write().await.user_login(ADMIN, ADMIN).await?;
    leader.client.read().await.admin_shutdown().await?;
    assert!(leader.server.process.wait()?.success());

    let mut statuses = Vec::with_capacity(2);

    for has_leader in servers
        .iter()
        .map(|s| tokio::spawn(wait_for_leader(s.client.clone())))
    {
        statuses.push(has_leader.await??);
    }

    for status in &statuses[1..] {
        assert_eq!(statuses[0], *status);
    }

    Ok(())
}

#[tokio::test]
async fn user_add() -> anyhow::Result<()> {
    let (leader, servers) = create_cluster(3).await?;
    leader.client.write().await.user_login(ADMIN, ADMIN).await?;
    leader
        .client
        .write()
        .await
        .admin_user_add("user1", "password123")
        .await?;

    for has_user in servers.iter().map(|s| {
        let client = s.client.clone();
        tokio::spawn(async move {
            client.write().await.user_login(ADMIN, ADMIN).await?;
            wait_for_user(client.clone(), "user1").await
        })
    }) {
        has_user.await??;
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
