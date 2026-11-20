#[path = "../../src/bin/agdb_harness/cluster.rs"]
mod cluster;
#[path = "../../src/bin/agdb_harness/server.rs"]
mod server;
pub mod test_cluster;
pub mod test_dir;
#[path = "../../src/bin/agdb_harness/test_error.rs"]
pub mod test_error;

pub use server::ADMIN;
pub use server::CONFIG_FILE;
pub use server::SERVER_DATA_DIR;
pub use server::TestServerImpl;
pub use server::api_for_test;
pub use server::reqwest_client;
pub use server::test_agent_name;

use agdb_api::AgdbApi;
use agdb_api::QueryAudit;
use agdb_api::ReqwestClient;
use agdb_api::config_impl::DEFAULT_CLUSTER_MAX_LOG_ENTRIES;
use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering;
use std::time::Duration;
use test_error::TestError;

const RETRY_TIMEOUT: Duration = Duration::from_secs(1);
const RETRY_ATTEMPS: u16 = 10;
static COUNTER: AtomicU16 = AtomicU16::new(1);
static SHARED: OnceLock<Shared> = OnceLock::new();

struct Shared {
    address: String,
    data_dir: String,
    cluster_addresses: Vec<String>,
    cluster_data_dirs: Vec<String>,
}

/// Starts the server and the cluster the tests share, they run until the
/// returned nodes are dropped.
pub async fn start() -> Result<Vec<TestServerImpl>, TestError> {
    let server = TestServerImpl::new().await?;
    let cluster = cluster::start_cluster(3, false, DEFAULT_CLUSTER_MAX_LOG_ENTRIES).await?;

    SHARED
        .set(Shared {
            address: server.address.clone(),
            data_dir: server.data_dir.clone(),
            cluster_addresses: cluster.iter().map(|node| node.address.clone()).collect(),
            cluster_data_dirs: cluster.iter().map(|node| node.data_dir.clone()).collect(),
        })
        .map_err(|_| TestError::new("the shared server is already started"))?;

    Ok(std::iter::once(server).chain(cluster).collect())
}

fn shared() -> &'static Shared {
    SHARED
        .get()
        .expect("the runner starts the shared server before the tests")
}

/// The shared server of the test run.
pub struct TestServer {
    pub data_dir: String,
    pub api: AgdbApi<ReqwestClient>,
}

impl TestServer {
    pub async fn new() -> Result<Self, TestError> {
        Ok(Self {
            data_dir: shared().data_dir.clone(),
            api: api_for_test(&shared().address),
        })
    }

    #[cfg(feature = "studio")]
    pub fn url(&self, uri: &str) -> String {
        format!("{}{uri}", self.api.address())
    }

    pub fn full_url(&self, uri: &str) -> String {
        self.api.base_url().to_string() + uri
    }
}

impl TestServerImpl {
    pub async fn new() -> Result<Self, TestError> {
        Self::with_config(server::default_config()).await
    }

    pub fn restart(&mut self) -> Result<(), TestError> {
        self.process = Some(server::spawn(&self.dir)?);
        Ok(())
    }

    pub async fn wait(&mut self) -> Result<(), TestError> {
        if let Some(p) = self.process.as_mut() {
            p.wait().await?;
        }

        Ok(())
    }
}

pub fn api_with_agent(address: &str, agent: &str) -> AgdbApi<ReqwestClient> {
    AgdbApi::new(
        ReqwestClient::with_user_agent(reqwest_client(), agent),
        address,
    )
}

pub fn next_user_name() -> String {
    format!(
        "db_user{}_{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    )
}

pub fn next_db_name() -> String {
    format!(
        "db{}_{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    )
}

pub fn audit_dir(data_dir: &str, owner: &str, db: &str) -> String {
    Path::new(data_dir)
        .join(owner)
        .join("audit")
        .join(db)
        .to_string_lossy()
        .to_string()
}

pub fn backup_audit_dir(data_dir: &str, owner: &str, db: &str) -> String {
    Path::new(data_dir)
        .join(owner)
        .join("backups")
        .join(format!("{db}.audit"))
        .to_string_lossy()
        .to_string()
}

pub fn audit_entries(path: &str) -> Result<usize, TestError> {
    let mut entries = 0;
    for file in std::fs::read_dir(path)? {
        let data = std::fs::read_to_string(file?.path())?;
        for line in data.lines().filter(|line| !line.is_empty()) {
            serde_json::from_str::<QueryAudit>(line).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("invalid audit json: {e}"),
                )
            })?;
            entries += 1;
        }
    }
    Ok(entries)
}

/// Appends an audit entry with the given `timestamp` to the day file
/// `{day}.log` (`YYYY-MM-DD`) of the audit directory `dir`.
pub fn write_audit_entry(dir: &str, day: &str, timestamp: u64) -> Result<(), TestError> {
    std::fs::create_dir_all(dir)?;
    let entry = QueryAudit {
        timestamp,
        username: ADMIN.to_string(),
        query: agdb::QueryBuilder::insert().nodes().count(1).query().into(),
    };
    let json = serde_json::to_string(&entry).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, format!("audit json: {e}"))
    })?;
    let line = format!("{json}\n");
    std::io::Write::write_all(
        &mut std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(Path::new(dir).join(format!("{day}.log")))?,
        line.as_bytes(),
    )?;
    Ok(())
}

pub async fn wait_for_ready(api: &AgdbApi<ReqwestClient>) -> Result<(), TestError> {
    for _ in 0..RETRY_ATTEMPS {
        if api.status().await.is_ok() {
            return Ok(());
        }

        std::thread::sleep(RETRY_TIMEOUT);
    }

    Err(TestError::new("Server not ready"))
}
