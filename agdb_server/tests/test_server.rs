#[path = "../src/config.rs"]
pub mod config;
mod routes;
#[cfg(feature = "tls")]
mod tls;

use crate::config::ConfigImpl;
use crate::config::DEFAULT_LOG_BODY_LIMIT;
use crate::config::DEFAULT_REQUEST_BODY_LIMIT;
use crate::config::to_str;
use agdb_api::AgdbApi;
use agdb_api::ClusterStatus;
use agdb_api::ReqwestClient;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Weak;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;
use tokio::process::Child;
use tokio::process::Command;
use tokio::sync::RwLock;
use tracing::level_filters::LevelFilter;

const ADMIN: &str = "admin";
const BINARY: &str = "agdb_server";
const CONFIG_FILE: &str = "agdb_server.yaml";
const DEFAULT_PORT: u16 = 3000;
const HOST: &str = "localhost";
const POLL_INTERVAL: u64 = 100;
const RETRY_TIMEOUT: Duration = Duration::from_secs(1);
const RETRY_ATTEMPS: u16 = 10;
const SERVER_DATA_DIR: &str = "agdb_server_data";
const SHUTDOWN_RETRY_TIMEOUT: Duration = Duration::from_millis(100);
const SHUTDOWN_RETRY_ATTEMPTS: u16 = 100;
const TEST_TIMEOUT: u128 = 30000;
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

type ClusterImpl = Vec<TestServerImpl>;

static PORT: AtomicU16 = AtomicU16::new(DEFAULT_PORT);
static COUNTER: AtomicU16 = AtomicU16::new(1);
static SERVER: std::sync::OnceLock<RwLock<Weak<TestServerImpl>>> = std::sync::OnceLock::new();
static CLUSTER: std::sync::OnceLock<RwLock<Weak<ClusterImpl>>> = std::sync::OnceLock::new();

fn server_bin() -> anyhow::Result<PathBuf> {
    let mut path = std::env::current_exe()?;
    path.pop();
    path.pop();
    Ok(path.join(format!("{BINARY}{}", std::env::consts::EXE_SUFFIX)))
}

pub struct TestServer {
    pub dir: String,
    pub data_dir: String,
    pub api: AgdbApi<ReqwestClient>,
    pub server: Arc<TestServerImpl>,
}

pub struct TestServerImpl {
    pub dir: String,
    pub data_dir: String,
    pub address: String,
    pub process: Option<Child>,
}

pub struct TestCluster {
    apis: Vec<AgdbApi<ReqwestClient>>,
    _cluster: Arc<ClusterImpl>,
}

#[cfg(feature = "tls")]
pub fn root_ca() -> reqwest::Certificate {
    static ROOT_CA: std::sync::OnceLock<reqwest::Certificate> = std::sync::OnceLock::new();

    ROOT_CA
        .get_or_init(|| {
            let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            let root_ca_buf =
                std::fs::read(format!("{manifest_dir}/tests/test_root_ca.pem")).unwrap();
            reqwest::Certificate::from_pem(&root_ca_buf).unwrap()
        })
        .clone()
}

#[cfg(feature = "tls")]
pub fn reqwest_client() -> reqwest::Client {
    reqwest::Client::builder()
        .add_root_certificate(root_ca())
        .use_rustls_tls()
        .timeout(CLIENT_TIMEOUT)
        .build()
        .unwrap()
}

#[cfg(not(feature = "tls"))]
pub fn reqwest_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(CLIENT_TIMEOUT)
        .build()
        .unwrap()
}

impl TestServerImpl {
    pub async fn with_config(mut config: ConfigImpl) -> anyhow::Result<Self> {
        if config.address.is_empty() {
            let port = Self::next_port();
            let address = format!("http://{HOST}:{port}");
            config.bind = format!("{HOST}:{port}");
            config.address = address;
        };

        let dir = format!(
            "{BINARY}.{}.test",
            config.address.split(':').last().unwrap()
        );
        let data_dir = format!("{dir}/{SERVER_DATA_DIR}");

        Self::remove_dir_if_exists(&dir)?;
        std::fs::create_dir(&dir)?;

        std::fs::write(Path::new(&dir).join(CONFIG_FILE), to_str(&config))?;

        let api_address = if config.basepath.is_empty() {
            config.address.clone()
        } else {
            format!("{}{}", config.address, config.basepath)
        };

        let mut process = Command::new(server_bin()?)
            .current_dir(&dir)
            .kill_on_drop(true)
            .spawn()?;
        let api = AgdbApi::new(ReqwestClient::with_client(reqwest_client()), &api_address);

        for _ in 0..RETRY_ATTEMPS {
            match api.status().await {
                Ok(200) => {
                    return Ok(Self {
                        dir,
                        data_dir,
                        address: api_address,
                        process: Some(process),
                    });
                }
                Ok(status) => println!("Server at {api_address} is not ready: {status}"),
                Err(e) => println!("Failed to contact server at {api_address}: {e:?}"),
            }

            std::thread::sleep(RETRY_TIMEOUT);
        }

        let mut status = "running".to_string();
        if let Ok(Some(s)) = process.try_wait() {
            if let Some(code) = s.code() {
                status = code.to_string()
            }
        }

        anyhow::bail!("Failed to start server '{api_address}' ({status})")
    }

    pub async fn new() -> anyhow::Result<Self> {
        let config = ConfigImpl {
            bind: String::new(),
            address: String::new(),
            basepath: String::new(),
            admin: ADMIN.to_string(),
            log_level: LevelFilter::INFO,
            log_body_limit: DEFAULT_LOG_BODY_LIMIT,
            request_body_limit: DEFAULT_REQUEST_BODY_LIMIT,
            data_dir: SERVER_DATA_DIR.into(),
            pepper_path: String::new(),
            tls_certificate: String::new(),
            tls_key: String::new(),
            tls_root: String::new(),
            cluster_token: "test".to_string(),
            cluster_heartbeat_timeout_ms: 1000,
            cluster_term_timeout_ms: 3000,
            cluster: Vec::new(),
            cluster_node_id: 0,
            start_time: 0,
            pepper: None,
        };

        Self::with_config(config).await
    }

    pub fn next_port() -> u16 {
        PORT.fetch_add(1, Ordering::Relaxed) + std::process::id() as u16
    }

    pub fn restart(&mut self) -> anyhow::Result<()> {
        self.process = Some(
            Command::new(server_bin()?)
                .current_dir(&self.dir)
                .kill_on_drop(true)
                .spawn()?,
        );
        Ok(())
    }

    pub async fn wait(&mut self) -> anyhow::Result<()> {
        if let Some(p) = self.process.as_mut() {
            p.wait().await?;
        }

        Ok(())
    }

    async fn shutdown_server(mut process: Child, mut address: String) -> anyhow::Result<()> {
        if process.try_wait()?.is_some() {
            return Ok(());
        }

        if !address.starts_with("http") {
            address = format!("http://{}", address);
        }

        let mut admin = HashMap::<&str, String>::new();
        admin.insert("username", ADMIN.to_string());
        admin.insert("password", ADMIN.to_string());

        let client = reqwest_client();

        let token: String = client
            .post(format!("{}/api/v1/user/login", address))
            .json(&admin)
            .timeout(CLIENT_TIMEOUT)
            .send()
            .await?
            .json()
            .await?;

        client
            .post(format!("{}/api/v1/admin/shutdown", address))
            .timeout(CLIENT_TIMEOUT)
            .bearer_auth(token)
            .send()
            .await?;

        for _ in 0..SHUTDOWN_RETRY_ATTEMPTS {
            if process.try_wait()?.is_some() {
                return Ok(());
            }
            std::thread::sleep(SHUTDOWN_RETRY_TIMEOUT);
        }

        process.kill().await?;

        for _ in 0..SHUTDOWN_RETRY_ATTEMPTS {
            if process.try_wait()?.is_some() {
                return Ok(());
            }
            std::thread::sleep(SHUTDOWN_RETRY_TIMEOUT);
        }

        anyhow::bail!("Failed to shutdown server")
    }

    fn remove_dir_if_exists(dir: &str) -> anyhow::Result<()> {
        if Path::new(dir).exists() {
            std::fs::remove_dir_all(dir)?;
        }

        Ok(())
    }
}

impl TestServer {
    pub async fn new() -> anyhow::Result<Self> {
        let global_server = SERVER.get_or_init(|| RwLock::new(Weak::new()));
        let mut server_guard = global_server.write().await;

        let server = if let Some(server) = server_guard.upgrade() {
            server
        } else {
            let server = Arc::new(TestServerImpl::new().await?);
            *server_guard = Arc::downgrade(&server);
            server
        };

        Ok(Self {
            api: AgdbApi::new(
                ReqwestClient::with_client(reqwest_client()),
                &server.address,
            ),
            dir: server.dir.clone(),
            data_dir: server.data_dir.clone(),
            server,
        })
    }

    pub fn url(&self, uri: &str) -> String {
        format!("{}{uri}", self.api.address())
    }

    pub fn full_url(&self, uri: &str) -> String {
        self.api.base_url().to_string() + uri
    }
}

impl Drop for TestServerImpl {
    fn drop(&mut self) {
        static DROP_RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> =
            std::sync::OnceLock::new();

        if let Some(p) = self.process.take() {
            let address = self.address.clone();
            let dir = self.dir.clone();

            let f = DROP_RUNTIME
                .get_or_init(|| tokio::runtime::Runtime::new().unwrap())
                .spawn(async move {
                    let _ = Self::shutdown_server(p, address)
                        .await
                        .inspect_err(|e| println!("{e:?}"));
                });

            for _ in 0..SHUTDOWN_RETRY_ATTEMPTS {
                if f.is_finished() {
                    break;
                }

                std::thread::sleep(SHUTDOWN_RETRY_TIMEOUT);
            }

            let _ = Self::remove_dir_if_exists(&dir).inspect_err(|e| println!("{e:?}"));
        }
    }
}

impl TestCluster {
    async fn new() -> anyhow::Result<Self> {
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
                .collect::<anyhow::Result<Vec<AgdbApi<ReqwestClient>>>>()?,
            _cluster: nodes,
        };

        cluster.apis[1].cluster_user_login(ADMIN, ADMIN).await?;

        Ok(cluster)
    }
}

pub fn next_user_name() -> String {
    format!("db_user{}", COUNTER.fetch_add(1, Ordering::SeqCst))
}

pub fn next_db_name() -> String {
    format!("db{}", COUNTER.fetch_add(1, Ordering::SeqCst))
}

pub async fn wait_for_ready(api: &AgdbApi<ReqwestClient>) -> anyhow::Result<()> {
    for _ in 0..RETRY_ATTEMPS {
        if api.status().await.is_ok() {
            return Ok(());
        }

        std::thread::sleep(RETRY_TIMEOUT);
    }

    anyhow::bail!("Server not ready")
}

pub async fn wait_for_leader(api: &AgdbApi<ReqwestClient>) -> anyhow::Result<Vec<ClusterStatus>> {
    let now = Instant::now();

    while now.elapsed().as_millis() < TEST_TIMEOUT {
        let status = api.cluster_status().await?;

        if status.1.iter().any(|s| s.leader) {
            return Ok(status.1);
        }

        std::thread::sleep(std::time::Duration::from_millis(POLL_INTERVAL));
    }

    Err(anyhow::anyhow!(
        "Leader not found within {TEST_TIMEOUT}seconds"
    ))
}

pub async fn create_cluster(nodes: usize, tls: bool) -> anyhow::Result<Vec<TestServerImpl>> {
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
            admin: ADMIN.to_string(),
            log_level: LevelFilter::INFO,
            log_body_limit: DEFAULT_LOG_BODY_LIMIT,
            request_body_limit: DEFAULT_REQUEST_BODY_LIMIT,
            data_dir: SERVER_DATA_DIR.into(),
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
        .find_map(|(i, s)| if s.leader { Some(i) } else { None })
        .unwrap();
    servers.swap(0, leader);

    Ok(servers.into_iter().map(|(s, _)| s).collect())
}
