use crate::AgdbApi;
use crate::ClusterStatus;
use crate::ReqwestClient;
use crate::config_impl::ConfigImpl;
use crate::test_server::ADMIN;
use crate::test_server::HOST;
use crate::test_server::POLL_INTERVAL;
use crate::test_server::TEST_TIMEOUT;
use crate::test_server::TestServerImpl;
use crate::test_server::reqwest_client;
use crate::test_server::test_error::TestError;
use crate::test_server::test_error::bail;
use std::sync::Arc;
use std::sync::Weak;
use std::time::Instant;
use tokio::sync::RwLock;

type ClusterImpl = Vec<TestServerImpl>;

static CLUSTER: std::sync::OnceLock<RwLock<Weak<ClusterImpl>>> = std::sync::OnceLock::new();

#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub struct TestCluster {
    pub apis: Vec<AgdbApi<ReqwestClient>>,
    pub cluster: Arc<ClusterImpl>,
}

#[cfg_attr(feature = "api", agdb::impl_def())]
impl TestCluster {
    pub async fn new() -> Result<Self, TestError> {
        let global_cluster = CLUSTER.get_or_init(|| RwLock::new(Weak::new()));
        let mut cluster_guard = global_cluster.write().await;

        let nodes = if let Some(nodes) = cluster_guard.upgrade() {
            nodes
        } else {
            let nodes = Arc::new(create_cluster(3, false).await?);
            *cluster_guard = Arc::downgrade(&nodes);
            nodes
        };

        let mut cluster = Self {
            apis: nodes
                .iter()
                .map(|s| {
                    Ok(AgdbApi::new(
                        ReqwestClient::with_client(reqwest_client()),
                        &s.address,
                    ))
                })
                .collect::<Result<Vec<AgdbApi<ReqwestClient>>, TestError>>()?,
            cluster: nodes,
        };

        cluster.apis[1].cluster_user_login(ADMIN, ADMIN).await?;

        Ok(cluster)
    }
}

#[cfg_attr(feature = "api", agdb::fn_def())]
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

    bail!("Leader not found within {TEST_TIMEOUT}seconds")
}

#[cfg_attr(feature = "api", agdb::fn_def())]
pub async fn create_cluster(nodes: usize, tls: bool) -> Result<Vec<TestServerImpl>, TestError> {
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
        let port = TestServerImpl::next_port();
        let config = ConfigImpl {
            bind: format!("{HOST}:{port}"),
            address: format!("{protocol}://{HOST}:{port}"),
            basepath: String::new(),
            static_roots: Vec::new(),
            admin: ADMIN.to_string(),
            log_level: crate::LogLevelFilter::Info,
            log_body_limit: crate::config_impl::DEFAULT_LOG_BODY_LIMIT,
            request_body_limit: crate::config_impl::DEFAULT_REQUEST_BODY_LIMIT,
            data_dir: super::SERVER_DATA_DIR.into(),
            pepper_path: String::new(),
            tls_certificate: tls_cert.clone(),
            tls_key: tls_key.clone(),
            tls_root: tls_root.clone(),
            cluster_token: "test".to_string(),
            cluster_heartbeat_timeout_ms: 1000,
            cluster_term_timeout_ms: 3000,
            cluster: Vec::new(),
            cluster_node_id: 0,
            start_time: 0,
            pepper: None,
        };

        configs.push(config);
        cluster.push(format!("{protocol}://{HOST}:{port}"));
    }

    for config in &mut configs {
        config.cluster = cluster.clone();
    }

    for server in configs
        .into_iter()
        .map(|c| tokio::spawn(async move { TestServerImpl::with_config(c).await }))
    {
        let server = server.await??;
        let api = AgdbApi::new(
            ReqwestClient::with_client(reqwest_client()),
            &server.address,
        );
        servers.push((server, api));
    }

    let mut statuses = Vec::with_capacity(nodes);

    for server in &servers {
        statuses.push(wait_for_leader(&server.1).await?);
    }

    for status in &statuses[1..] {
        assert_eq!(statuses[0], *status);
    }

    let leader = statuses[0]
        .iter()
        .enumerate()
        .find_map(|x| if x.1.leader { Some(x.0) } else { None })
        .unwrap();
    servers.swap(0, leader);

    Ok(servers.into_iter().map(|x| x.0).collect())
}
