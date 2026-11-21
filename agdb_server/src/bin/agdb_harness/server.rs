use super::test_error::TestError;
use agdb_api::AgdbApi;
use agdb_api::LogLevelFilter;
use agdb_api::ReqwestClient;
use agdb_api::config_impl::ConfigImpl;
use agdb_api::config_impl::DEFAULT_CLUSTER_MAX_APPEND_SIZE;
use agdb_api::config_impl::DEFAULT_CLUSTER_MAX_CHUNK_SIZE;
use agdb_api::config_impl::DEFAULT_CLUSTER_MAX_LOG_ENTRIES;
use agdb_api::config_impl::DEFAULT_LOG_BODY_LIMIT;
use agdb_api::config_impl::DEFAULT_REQUEST_BODY_LIMIT;
use agdb_api::config_impl::DEFAULT_TOKEN_EXPIRY_SECONDS;
use agdb_api::config_impl::config_to_str;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::sync::Mutex;
use std::time::Duration;
use tokio::process::Child;
use tokio::process::Command;

pub const ADMIN: &str = "admin";
pub const CONFIG_FILE: &str = "agdb_server.yaml";
pub const SERVER_DATA_DIR: &str = "agdb_server_data";
pub const HOST: &str = "localhost";
const BINARY: &str = "agdb_server";
const RETRY_TIMEOUT: Duration = Duration::from_secs(1);
const RETRY_ATTEMPS: u16 = 10;
const START_ATTEMPTS: u16 = 3;
const SHUTDOWN_RETRY_TIMEOUT: Duration = Duration::from_millis(100);
const SHUTDOWN_RETRY_ATTEMPTS: u16 = 100;
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

pub struct TestServerImpl {
    pub dir: String,
    pub data_dir: String,
    pub address: String,
    pub process: Option<Child>,
}

fn server_bin() -> Result<PathBuf, TestError> {
    // test executables are in `<profile>/deps` or, since Rust 1.100, `<profile>/build/<package>/<hash>/out`
    let exe = std::env::current_exe()?;
    let name = format!("{BINARY}{}", std::env::consts::EXE_SUFFIX);
    exe.ancestors()
        .skip(1)
        .map(|dir| dir.join(&name))
        .find(|path| path.is_file())
        .ok_or_else(|| {
            TestError::new(format!(
                "{name} not found above {}, build it first: cargo build -p agdb_server",
                exe.display()
            ))
        })
}

/// Starts the server binary in `dir`, the server reads its config from there.
pub fn spawn(dir: &str) -> Result<Child, TestError> {
    Ok(Command::new(server_bin()?)
        .current_dir(dir)
        .kill_on_drop(true)
        .spawn()?)
}

pub fn server_dir(port: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{BINARY}.{port}.test"))
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

pub fn default_config() -> ConfigImpl {
    ConfigImpl {
        bind: String::new(),
        address: String::new(),
        basepath: String::new(),
        static_roots: Vec::new(),
        admin: ADMIN.to_string(),
        log_level: LogLevelFilter::Info,
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
        cluster_election_factor_ms: 1000,
        cluster: Vec::new(),
        cluster_max_log_entries: DEFAULT_CLUSTER_MAX_LOG_ENTRIES,
        cluster_max_chunk_size: DEFAULT_CLUSTER_MAX_CHUNK_SIZE,
        cluster_max_append_size: DEFAULT_CLUSTER_MAX_APPEND_SIZE,
        cluster_node_id: 0,
        start_time: 0,
        token_expiry_seconds: DEFAULT_TOKEN_EXPIRY_SECONDS,
        storage: agdb::StorageConfig::default(),
        pepper: None,
    }
}

enum Start {
    Running(Box<TestServerImpl>),
    Exited(ExitStatus),
}

impl TestServerImpl {
    /// Without an address in the `config` the server gets a free port, and
    /// another one if it exits before it answers (e.g. the port was taken).
    pub async fn with_config(mut config: ConfigImpl) -> Result<Self, TestError> {
        let pick_port = config.address.is_empty();

        for _ in 0..START_ATTEMPTS {
            if pick_port {
                let port = Self::next_port()?;
                config.bind = format!("{HOST}:{port}");
                config.address = format!("http://{HOST}:{port}");
            }

            match Self::start(&config).await? {
                Start::Running(server) => return Ok(*server),
                Start::Exited(status) if pick_port => {
                    println!(
                        "Server '{}' exited ({status}), retrying",
                        config.server_url()
                    );
                }
                Start::Exited(status) => {
                    return Err(TestError::new(format!(
                        "Server '{}' exited ({status}) before it answered",
                        config.server_url()
                    )));
                }
            }
        }

        Err(TestError::new(format!(
            "Server exited {START_ATTEMPTS} times before it answered"
        )))
    }

    /// The status of a server answering on a port that its process does not
    /// hold comes from another server, so the process must still run.
    async fn start(config: &ConfigImpl) -> Result<Start, TestError> {
        let dir = server_dir(config.address.split(':').next_back().unwrap())
            .to_string_lossy()
            .to_string();
        let data_dir = Path::new(&dir)
            .join(SERVER_DATA_DIR)
            .to_string_lossy()
            .to_string();

        Self::remove_dir_if_exists(&dir)?;
        std::fs::create_dir(&dir)?;
        std::fs::write(Path::new(&dir).join(CONFIG_FILE), config_to_str(config))?;

        let api_address = config.server_url();

        let mut process = spawn(&dir)?;
        let api = api_for_test(&api_address);

        for _ in 0..RETRY_ATTEMPS {
            let status = api.status().await;

            if let Some(exit) = process.try_wait()? {
                Self::remove_dir_if_exists(&dir)?;
                return Ok(Start::Exited(exit));
            }

            match status {
                Ok(200) => {
                    return Ok(Start::Running(Box::new(Self {
                        dir,
                        data_dir,
                        address: api_address,
                        process: Some(process),
                    })));
                }
                Ok(status) => println!("Server at {api_address} is not ready: {status}"),
                Err(e) => println!("Failed to contact server at {api_address}: {e:?}"),
            }

            std::thread::sleep(RETRY_TIMEOUT);
        }

        Err(TestError::new(format!(
            "Failed to start server '{api_address}' (running)"
        )))
    }

    /// A port the OS reports free, never the same twice in this process.
    pub fn next_port() -> Result<u16, TestError> {
        static PORTS: Mutex<BTreeSet<u16>> = Mutex::new(BTreeSet::new());

        loop {
            let port = std::net::TcpListener::bind((HOST, 0))?.local_addr()?.port();

            if PORTS
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(port)
            {
                return Ok(port);
            }
        }
    }

    async fn shutdown_server(mut process: Child, mut address: String) -> Result<(), TestError> {
        if process.try_wait()?.is_some() {
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

        Err(TestError::new("Failed to shutdown server"))
    }

    fn remove_dir_if_exists(dir: &str) -> Result<(), TestError> {
        if Path::new(dir).exists() {
            std::fs::remove_dir_all(dir)?;
        }

        Ok(())
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
