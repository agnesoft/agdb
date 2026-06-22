use crate::AgdbApi;
use crate::ClusterStatus;
use crate::LogLevelFilter;
use crate::ReqwestClient;
use crate::config_impl::ConfigImpl;
use crate::config_impl::DEFAULT_LOG_BODY_LIMIT;
use crate::config_impl::DEFAULT_REQUEST_BODY_LIMIT;
use crate::config_impl::DEFAULT_TOKEN_EXPIRY_SECONDS;
use crate::test_server::ADMIN;
use crate::test_server::HOST;
use crate::test_server::POLL_INTERVAL;
use crate::test_server::TEST_TIMEOUT;
use crate::test_server::TestServerImpl;
use crate::test_server::api_for_test;
use crate::test_server::test_error::TestError;
use crate::test_server::test_error::bail;
#[cfg(feature = "api")]
use agdb::type_def::Type;
#[cfg(feature = "api")]
use agdb::type_def::TypeDefinition;
use std::path::Path;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::Weak;
use std::time::Instant;
use tokio::sync::RwLock;

#[cfg_attr(feature = "api", agdb::static_def())]
static CLUSTER: OnceLock<RwLock<Weak<Vec<TestServerImpl>>>> = OnceLock::new();

#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
#[type_def(inherent)]
pub struct TestCluster {
    cluster: Arc<Vec<TestServerImpl>>,
}

#[cfg_attr(feature = "api", agdb::impl_def())]
impl TestCluster {
    pub async fn new() -> Result<Self, TestError> {
        let global = CLUSTER.get_or_init(|| RwLock::new(Weak::new()));
        let mut guard = global.write().await;
        let nodes = if let Some(nodes) = guard.upgrade() {
            nodes
        } else {
            let nodes = Arc::new(create_cluster(3, false).await?);
            *guard = Arc::downgrade(&nodes);
            nodes
        };
        drop(guard);

        Ok(Self { cluster: nodes })
    }

    pub async fn new_private() -> Result<Self, TestError> {
        let nodes = Arc::new(create_cluster(3, false).await?);
        Ok(Self { cluster: nodes })
    }

    pub fn leader(&self) -> AgdbApi<ReqwestClient> {
        api_for_test(&self.cluster[0].address)
    }

    pub fn follower(&self) -> AgdbApi<ReqwestClient> {
        api_for_test(&self.cluster[1].address)
    }
}

#[cfg_attr(feature = "api", agdb::fn_def())]
pub fn cluster_data_dir(address: &str) -> String {
    let without_scheme = address.split("://").nth(1).unwrap_or(address);
    let authority = without_scheme.split('/').next().unwrap_or(without_scheme);
    let port = authority.rsplit(':').next().unwrap_or(authority);
    Path::new(&format!("agdb_server.{port}.test"))
        .join(super::SERVER_DATA_DIR)
        .to_string_lossy()
        .to_string()
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
            basepath: "/base".to_string(),
            static_roots: Vec::new(),
            admin: ADMIN.to_string(),
            log_level: LogLevelFilter::Info,
            log_body_limit: DEFAULT_LOG_BODY_LIMIT,
            request_body_limit: DEFAULT_REQUEST_BODY_LIMIT,
            data_dir: super::SERVER_DATA_DIR.into(),
            pepper_path: String::new(),
            tls_certificate: tls_cert.clone(),
            tls_key: tls_key.clone(),
            tls_root: tls_root.clone(),
            cluster_token: "test".to_string(),
            cluster_heartbeat_timeout_ms: 1000,
            cluster_term_timeout_ms: 3000,
            cluster_election_factor_ms: 1000,
            cluster: Vec::new(),
            cluster_node_id: 0,
            start_time: 0,
            token_expiry_seconds: DEFAULT_TOKEN_EXPIRY_SECONDS,
            pepper: None,
        };

        configs.push(config);
        cluster.push(format!("{protocol}://{HOST}:{port}/base"));
    }

    for config in &mut configs {
        config.cluster = cluster.clone();
    }

    for server in configs
        .into_iter()
        .map(|c| tokio::spawn(async move { TestServerImpl::with_config(c).await }))
    {
        let server = server.await??;
        let api = api_for_test(&server.address);
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

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<Type> {
    vec![
        __CLUSTER_type_def(),
        TestCluster::type_def(),
        __wait_for_leader_type_def(),
        __create_cluster_type_def(),
        __cluster_data_dir_type_def(),
    ]
}
