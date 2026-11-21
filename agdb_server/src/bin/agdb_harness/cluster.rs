use super::server::ADMIN;
use super::server::HOST;
use super::server::SERVER_DATA_DIR;
use super::server::TestServerImpl;
use super::server::api_for_test;
use super::test_error::TestError;
use agdb_api::AgdbApi;
use agdb_api::ClusterStatus;
use agdb_api::LogLevelFilter;
use agdb_api::ReqwestClient;
use agdb_api::config_impl::ConfigImpl;
use agdb_api::config_impl::DEFAULT_CLUSTER_MAX_APPEND_SIZE;
use agdb_api::config_impl::DEFAULT_CLUSTER_MAX_CHUNK_SIZE;
use agdb_api::config_impl::DEFAULT_LOG_BODY_LIMIT;
use agdb_api::config_impl::DEFAULT_REQUEST_BODY_LIMIT;
use agdb_api::config_impl::DEFAULT_TOKEN_EXPIRY_SECONDS;
use std::time::Instant;
use tokio::sync::Semaphore;

const POLL_INTERVAL: u64 = 100;
const TEST_TIMEOUT: u128 = 30000;
static STARTING: Semaphore = Semaphore::const_new(4);

pub async fn wait_for_leader(
    api: &AgdbApi<ReqwestClient>,
) -> Result<Vec<ClusterStatus>, TestError> {
    let now = Instant::now();

    while now.elapsed().as_millis() < TEST_TIMEOUT {
        let status = api.cluster_status().await?;

        if status.1.iter().any(|s| s.leader) {
            return Ok(status.1);
        }

        std::thread::sleep(std::time::Duration::from_millis(POLL_INTERVAL));
    }

    Err(TestError::new(format!(
        "Leader not found within {TEST_TIMEOUT}seconds"
    )))
}

/// Every node reports all nodes up and the same leader. Right after the start the
/// nodes still learn about each other, so one snapshot of each can disagree.
async fn wait_for_agreement(
    apis: &[&AgdbApi<ReqwestClient>],
) -> Result<Vec<ClusterStatus>, TestError> {
    let now = Instant::now();

    while now.elapsed().as_millis() < TEST_TIMEOUT {
        let mut statuses = Vec::with_capacity(apis.len());

        for api in apis {
            statuses.push(wait_for_leader(api).await?);
        }

        if statuses
            .iter()
            .all(|status| *status == statuses[0] && status.iter().all(|node| node.status))
        {
            return Ok(statuses.swap_remove(0));
        }

        std::thread::sleep(std::time::Duration::from_millis(POLL_INTERVAL));
    }

    Err(TestError::new(format!(
        "Cluster nodes did not agree within {TEST_TIMEOUT} ms"
    )))
}

pub async fn start_cluster(
    nodes: usize,
    tls: bool,
    max_log_entries: u64,
) -> Result<Vec<TestServerImpl>, TestError> {
    let mut configs = Vec::with_capacity(nodes);
    let mut cluster = Vec::with_capacity(nodes);
    let mut servers = Vec::with_capacity(nodes);
    let protocol = if tls { "https" } else { "http" };
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let tls_cert = if tls {
        format!("{manifest_dir}/tests/test_cert.pem")
    } else {
        String::new()
    };
    let tls_key = if tls {
        format!("{manifest_dir}/tests/test_cert.key.pem")
    } else {
        String::new()
    };
    let tls_root = if tls {
        format!("{manifest_dir}/tests/test_root_ca.pem")
    } else {
        String::new()
    };

    for _ in 0..nodes {
        let port = TestServerImpl::next_port()?;
        let config = ConfigImpl {
            bind: format!("{HOST}:{port}"),
            address: format!("{protocol}://{HOST}:{port}"),
            basepath: "/base".to_string(),
            static_roots: Vec::new(),
            admin: ADMIN.to_string(),
            log_level: LogLevelFilter::Info,
            log_body_limit: DEFAULT_LOG_BODY_LIMIT,
            request_body_limit: DEFAULT_REQUEST_BODY_LIMIT,
            data_dir: SERVER_DATA_DIR.into(),
            pepper_path: String::new(),
            tls_certificate: tls_cert.clone(),
            tls_key: tls_key.clone(),
            tls_root: tls_root.clone(),
            cluster_token: "test".to_string(),
            cluster_heartbeat_timeout_ms: 250,
            cluster_term_timeout_ms: 1000,
            cluster_election_factor_ms: 250,
            cluster: Vec::new(),
            cluster_max_log_entries: max_log_entries,
            cluster_max_chunk_size: DEFAULT_CLUSTER_MAX_CHUNK_SIZE,
            cluster_max_append_size: DEFAULT_CLUSTER_MAX_APPEND_SIZE,
            cluster_node_id: 0,
            start_time: 0,
            token_expiry_seconds: DEFAULT_TOKEN_EXPIRY_SECONDS,
            storage: agdb::StorageConfig::default(),
            pepper: None,
        };

        configs.push(config);
        cluster.push(format!("{protocol}://{HOST}:{port}/base"));
    }

    for config in &mut configs {
        config.cluster = cluster.clone();
    }

    // Many clusters electing at once starve each other into re-elections, and the
    // tests expect the leader they were given to stay leader.
    let _starting = STARTING
        .acquire()
        .await
        .map_err(|e| TestError::new(e.to_string()))?;

    for server in configs
        .into_iter()
        .map(|c| tokio::spawn(async move { TestServerImpl::with_config(c).await }))
    {
        let server = server.await??;
        let api = api_for_test(&server.address);
        servers.push((server, api));
    }

    let apis: Vec<&AgdbApi<ReqwestClient>> = servers.iter().map(|server| &server.1).collect();
    // The statuses are sorted by address, not in the order of the nodes.
    let statuses = wait_for_agreement(&apis).await?;
    let leader = statuses
        .iter()
        .find(|status| status.leader)
        .and_then(|status| {
            servers
                .iter()
                .position(|server| server.0.address == status.address)
        })
        .ok_or_else(|| TestError::new(format!("Leader is not a started node: {statuses:?}")))?;
    servers.swap(0, leader);

    Ok(servers.into_iter().map(|x| x.0).collect())
}
