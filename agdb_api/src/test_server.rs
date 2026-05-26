pub mod test_cluster;
pub mod test_dir;
pub mod test_error;

use crate::AgdbApi;
use crate::QueryAudit;
use crate::ReqwestClient;
use crate::config_impl::ConfigImpl;
use crate::config_impl::DEFAULT_LOG_BODY_LIMIT;
use crate::config_impl::DEFAULT_REQUEST_BODY_LIMIT;
use crate::config_impl::DEFAULT_TOKEN_EXPIRY_SECONDS;
use crate::config_impl::config_to_str;
use crate::test_server::test_error::TestError;
use crate::test_server::test_error::bail;
#[cfg(feature = "api")]
use agdb::type_def::TypeDefinition;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Weak;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::process::Child;
use tokio::process::Command;
use tokio::sync::RwLock;

#[cfg_attr(feature = "api", agdb::static_def())]
pub const ADMIN: &str = "admin";
#[cfg_attr(feature = "api", agdb::static_def())]
pub const CONFIG_FILE: &str = "agdb_server.yaml";
#[cfg_attr(feature = "api", agdb::static_def())]
pub const SERVER_DATA_DIR: &str = "agdb_server_data";
#[cfg_attr(feature = "api", agdb::static_def())]
pub(crate) const HOST: &str = "localhost";
#[cfg_attr(feature = "api", agdb::static_def())]
const BINARY: &str = "agdb_server";
#[cfg_attr(feature = "api", agdb::static_def())]
const DEFAULT_PORT: u16 = 3000;
#[cfg_attr(feature = "api", agdb::static_def())]
const POLL_INTERVAL: u64 = 100;
#[cfg_attr(feature = "api", agdb::static_def())]
const RETRY_TIMEOUT: Duration = Duration::from_secs(1);
#[cfg_attr(feature = "api", agdb::static_def())]
const RETRY_ATTEMPS: u16 = 10;
#[cfg_attr(feature = "api", agdb::static_def())]
const SHUTDOWN_RETRY_TIMEOUT: Duration = Duration::from_millis(100);
#[cfg_attr(feature = "api", agdb::static_def())]
const SHUTDOWN_RETRY_ATTEMPTS: u16 = 100;
#[cfg_attr(feature = "api", agdb::static_def())]
const TEST_TIMEOUT: u128 = 30000;
#[cfg_attr(feature = "api", agdb::static_def())]
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);
#[cfg_attr(feature = "api", agdb::static_def())]
static PORT: AtomicU16 = AtomicU16::new(DEFAULT_PORT);
#[cfg_attr(feature = "api", agdb::static_def())]
static COUNTER: AtomicU16 = AtomicU16::new(1);
#[cfg_attr(feature = "api", agdb::static_def())]
static SERVER: std::sync::OnceLock<RwLock<Weak<TestServerImpl>>> = std::sync::OnceLock::new();

pub struct TestServerProcess(pub Child);

#[cfg(feature = "api")]
impl agdb::type_def::TypeDefinition for TestServerProcess {
    fn type_def() -> agdb::type_def::Type {
        agdb::type_def::Type::Struct(agdb::type_def::Struct {
            name: "TestServerProcess",
            generics: &[],
            fields: &[],
            impl_defs: Vec::new,
        })
    }
}

#[cfg_attr(feature = "api", agdb::fn_def())]
fn server_bin() -> Result<PathBuf, TestError> {
    let mut path = std::env::current_exe()?;
    path.pop();
    path.pop();
    Ok(path.join(format!("{BINARY}{}", std::env::consts::EXE_SUFFIX)))
}

#[cfg_attr(feature = "api", agdb::fn_def())]
pub fn next_user_name() -> String {
    format!("db_user{}", COUNTER.fetch_add(1, Ordering::SeqCst))
}

#[cfg_attr(feature = "api", agdb::fn_def())]
pub fn next_db_name() -> String {
    format!("db{}", COUNTER.fetch_add(1, Ordering::SeqCst))
}

#[cfg_attr(feature = "api", agdb::fn_def())]
pub fn audit_file(data_dir: &str, owner: &str, db: &str) -> String {
    Path::new(data_dir)
        .join(owner)
        .join("audit")
        .join(format!("{db}.log"))
        .to_string_lossy()
        .to_string()
}

#[cfg_attr(feature = "api", agdb::fn_def())]
pub fn backup_audit_file(data_dir: &str, owner: &str, db: &str) -> String {
    Path::new(data_dir)
        .join(owner)
        .join("backups")
        .join(format!("{db}.log"))
        .to_string_lossy()
        .to_string()
}

#[cfg_attr(feature = "api", agdb::fn_def())]
pub fn audit_entries(path: &str) -> Result<usize, TestError> {
    let data = std::fs::read_to_string(path)?;
    let entries: Vec<QueryAudit> = serde_json::from_str(&data).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid audit json: {e}"),
        )
    })?;
    Ok(entries.len())
}

pub fn test_agent_name() -> String {
    std::env::var("NEXTEST_TEST_NAME")
        .ok()
        .filter(|name| !name.is_empty())
        .or_else(|| {
            std::thread::current()
                .name()
                .map(std::string::ToString::to_string)
                .filter(|name| !name.is_empty())
        })
        .unwrap_or_else(|| "agdb_api_test".to_string())
}

pub fn api_for_test(address: &str) -> AgdbApi<ReqwestClient> {
    AgdbApi::new(
        ReqwestClient::with_user_agent(reqwest_client(), test_agent_name()),
        address,
    )
}

#[cfg_attr(feature = "api", agdb::fn_def())]
pub async fn wait_for_ready(api: &AgdbApi<ReqwestClient>) -> Result<(), TestError> {
    for _ in 0..RETRY_ATTEMPS {
        if api.status().await.is_ok() {
            return Ok(());
        }

        std::thread::sleep(RETRY_TIMEOUT);
    }

    bail!("Server not ready")
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    let mut defs = vec![
        __ADMIN_type_def(),
        __CONFIG_FILE_type_def(),
        __SERVER_DATA_DIR_type_def(),
        __HOST_type_def(),
        __BINARY_type_def(),
        __DEFAULT_PORT_type_def(),
        __POLL_INTERVAL_type_def(),
        __RETRY_TIMEOUT_type_def(),
        __RETRY_ATTEMPS_type_def(),
        __SHUTDOWN_RETRY_TIMEOUT_type_def(),
        __SHUTDOWN_RETRY_ATTEMPTS_type_def(),
        __TEST_TIMEOUT_type_def(),
        __CLIENT_TIMEOUT_type_def(),
        __PORT_type_def(),
        __COUNTER_type_def(),
        __SERVER_type_def(),
        TestServerProcess::type_def(),
        __server_bin_type_def(),
        __next_user_name_type_def(),
        __next_db_name_type_def(),
        __audit_file_type_def(),
        __backup_audit_file_type_def(),
        __audit_entries_type_def(),
        __wait_for_ready_type_def(),
        TestError::type_def(),
        PathBuf::type_def(),
        TestServer::type_def(),
        TestServerImpl::type_def(),
    ];

    defs.extend(test_cluster::test_defs());

    defs
}

#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub struct TestServer {
    pub dir: String,
    pub data_dir: String,
    pub api: AgdbApi<ReqwestClient>,
    pub server: Arc<TestServerImpl>,
}

#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub struct TestServerImpl {
    pub dir: String,
    pub data_dir: String,
    pub address: String,
    pub process: Option<TestServerProcess>,
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

#[cfg_attr(feature = "api", agdb::impl_def())]
impl TestServerImpl {
    pub async fn with_config(mut config: ConfigImpl) -> Result<Self, TestError> {
        if config.address.is_empty() {
            let port = Self::next_port();
            let address = format!("http://{HOST}:{port}");
            config.bind = format!("{HOST}:{port}");
            config.address = address;
        };

        let dir = format!(
            "{BINARY}.{}.test",
            config.address.split(':').next_back().unwrap()
        );
        let data_dir = format!("{dir}/{SERVER_DATA_DIR}");

        Self::remove_dir_if_exists(&dir)?;
        std::fs::create_dir(&dir)?;
        std::fs::write(Path::new(&dir).join(CONFIG_FILE), config_to_str(&config))?;

        let api_address = if config.basepath.is_empty() {
            config.address.clone()
        } else {
            format!("{}{}", config.address, config.basepath)
        };

        let mut process = Command::new(server_bin()?)
            .current_dir(&dir)
            .kill_on_drop(true)
            .spawn()?;
        let api = api_for_test(&api_address);

        for _ in 0..RETRY_ATTEMPS {
            match api.status().await {
                Ok(200) => {
                    return Ok(Self {
                        dir,
                        data_dir,
                        address: api_address,
                        process: Some(TestServerProcess(process)),
                    });
                }
                Ok(status) => println!("Server at {api_address} is not ready: {status}"),
                Err(e) => println!("Failed to contact server at {api_address}: {e:?}"),
            }

            std::thread::sleep(RETRY_TIMEOUT);
        }

        let mut status = "running".to_string();
        if let Ok(Some(s)) = process.try_wait()
            && let Some(code) = s.code()
        {
            status = code.to_string()
        }

        bail!("Failed to start server '{api_address}' ({status})")
    }

    pub async fn new() -> Result<Self, TestError> {
        let config = ConfigImpl {
            bind: String::new(),
            address: String::new(),
            basepath: String::new(),
            static_roots: Vec::new(),
            admin: ADMIN.to_string(),
            log_level: crate::LogLevelFilter::Info,
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
            token_expiry_seconds: DEFAULT_TOKEN_EXPIRY_SECONDS,
            pepper: None,
        };

        Self::with_config(config).await
    }

    pub fn next_port() -> u16 {
        PORT.fetch_add(1, Ordering::Relaxed) + std::process::id() as u16
    }

    pub fn restart(&mut self) -> Result<(), TestError> {
        self.process = Some(TestServerProcess(
            Command::new(server_bin()?)
                .current_dir(&self.dir)
                .kill_on_drop(true)
                .spawn()?,
        ));
        Ok(())
    }

    pub async fn wait(&mut self) -> Result<(), TestError> {
        if let Some(p) = self.process.as_mut() {
            p.0.wait().await?;
        }

        Ok(())
    }

    async fn shutdown_server(
        mut process: TestServerProcess,
        mut address: String,
    ) -> Result<(), TestError> {
        if process.0.try_wait()?.is_some() {
            return Ok(());
        }

        if !address.starts_with("http") {
            address = format!("http://{address}");
        }

        let mut admin = HashMap::<&str, String>::new();
        admin.insert("username", ADMIN.to_string());
        admin.insert("password", ADMIN.to_string());

        let client = reqwest_client();

        let token: String = client
            .post(format!("{address}/api/v1/user/login"))
            .json(&admin)
            .timeout(CLIENT_TIMEOUT)
            .send()
            .await?
            .json()
            .await?;

        client
            .post(format!("{address}/api/v1/admin/shutdown"))
            .timeout(CLIENT_TIMEOUT)
            .bearer_auth(token)
            .send()
            .await?;

        for _ in 0..SHUTDOWN_RETRY_ATTEMPTS {
            if process.0.try_wait()?.is_some() {
                return Ok(());
            }
            std::thread::sleep(SHUTDOWN_RETRY_TIMEOUT);
        }

        process.0.kill().await?;

        for _ in 0..SHUTDOWN_RETRY_ATTEMPTS {
            if process.0.try_wait()?.is_some() {
                return Ok(());
            }
            std::thread::sleep(SHUTDOWN_RETRY_TIMEOUT);
        }

        bail!("Failed to shutdown server")
    }

    fn remove_dir_if_exists(dir: &str) -> Result<(), TestError> {
        if Path::new(dir).exists() {
            std::fs::remove_dir_all(dir)?;
        }

        Ok(())
    }
}

#[cfg_attr(feature = "api", agdb::impl_def())]
impl TestServer {
    pub async fn new() -> Result<Self, TestError> {
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
            api: api_for_test(&server.address),
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

#[cfg_attr(feature = "api", agdb::impl_def())]
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
