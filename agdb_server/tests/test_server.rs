mod routes;

use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;
use anyhow::anyhow;
use assert_cmd::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::process::Child;
use std::process::Command;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering;
use std::time::Duration;

const BINARY: &str = "agdb_server";
const CONFIG_FILE: &str = "agdb_server.yaml";
const SERVER_DATA_DIR: &str = "agdb_server_data";
const PROTOCOL: &str = "http";
const HOST: &str = "localhost";
const DEFAULT_PORT: u16 = 3000;
const ADMIN: &str = "admin";
const RETRY_TIMEOUT: Duration = Duration::from_secs(1);
const RETRY_ATTEMPS: u16 = 3;
const SHUTDOWN_RETRY_TIMEOUT: Duration = Duration::from_millis(100);
const SHUTDOWN_RETRY_ATTEMPTS: u16 = 100;

static PORT: AtomicU16 = AtomicU16::new(DEFAULT_PORT);
static COUNTER: AtomicU16 = AtomicU16::new(1);
static MUTEX: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();
static SERVER: std::sync::OnceLock<tokio::sync::RwLock<Option<TestServerImpl>>> =
    std::sync::OnceLock::new();

pub struct TestServer {
    pub dir: String,
    pub data_dir: String,
    pub port: u16,
    pub api: AgdbApi<ReqwestClient>,
}

struct TestServerImpl {
    pub dir: String,
    pub data_dir: String,
    pub port: u16,
    pub process: Child,
    pub instances: u16,
}

impl TestServerImpl {
    pub async fn new() -> anyhow::Result<Self> {
        let port = PORT.fetch_add(1, Ordering::Relaxed) + std::process::id() as u16;
        let dir = format!("{BINARY}.{port}.test");
        let data_dir = format!("{dir}/{SERVER_DATA_DIR}");

        Self::remove_dir_if_exists(&dir)?;
        std::fs::create_dir(&dir)?;

        let mut config = HashMap::<&str, serde_yaml::Value>::new();
        config.insert("host", HOST.into());
        config.insert("port", port.into());
        config.insert("admin", ADMIN.into());
        config.insert("data_dir", SERVER_DATA_DIR.into());

        let file = std::fs::File::options()
            .create_new(true)
            .write(true)
            .open(Path::new(&dir).join(CONFIG_FILE))?;
        serde_yaml::to_writer(file, &config)?;

        let process = Command::cargo_bin(BINARY)?.current_dir(&dir).spawn()?;
        let api = AgdbApi::new(ReqwestClient::new(), &TestServer::url_base(), port);

        for _ in 0..RETRY_ATTEMPS {
            if let Ok(status) = api.status().await {
                if status == 200 {
                    return Ok(Self {
                        dir,
                        data_dir,
                        port,
                        process,
                        instances: 1,
                    });
                }
            }

            std::thread::sleep(RETRY_TIMEOUT);
        }

        anyhow::bail!("Failed to start server")
    }

    fn shutdown_server(&mut self) -> anyhow::Result<()> {
        if self.process.try_wait()?.is_some() {
            return Ok(());
        }

        let port = self.port;
        let mut admin = HashMap::<&str, String>::new();
        admin.insert("username", ADMIN.to_string());
        admin.insert("password", ADMIN.to_string());

        std::thread::spawn(move || -> anyhow::Result<()> {
            let client = reqwest::blocking::Client::new();
            let token: String = client
                .post(format!(
                    "{}:{}/api/v1/user/login",
                    TestServer::url_base(),
                    port
                ))
                .json(&admin)
                .send()?
                .json()?;

            client
                .post(format!(
                    "{}:{}/api/v1/admin/shutdown",
                    TestServer::url_base(),
                    port
                ))
                .bearer_auth(token)
                .send()?;
            Ok(())
        })
        .join()
        .map_err(|e| anyhow!("{:?}", e))??;

        for _ in 0..SHUTDOWN_RETRY_ATTEMPTS {
            if self.process.try_wait()?.is_some() {
                return Ok(());
            }
            std::thread::sleep(SHUTDOWN_RETRY_TIMEOUT);
        }

        self.process.kill()?;
        self.process.wait()?;

        Ok(())
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
        let _guard = MUTEX
            .get_or_init(|| tokio::sync::Mutex::new(()))
            .lock()
            .await;
        let global_server = SERVER.get_or_init(|| tokio::sync::RwLock::new(None));
        let mut server_guard = global_server.try_write().unwrap();

        if server_guard.is_none() {
            *server_guard = Some(TestServerImpl::new().await?);
        } else {
            server_guard.as_mut().unwrap().instances += 1;
        }

        let server = server_guard.as_ref().unwrap();

        Ok(Self {
            api: AgdbApi::new(ReqwestClient::new(), &Self::url_base(), server.port),
            dir: server.dir.clone(),
            port: server.port,
            data_dir: server.data_dir.clone(),
        })
    }

    pub fn next_user_name(&mut self) -> String {
        format!("db_user{}", COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn next_db_name(&mut self) -> String {
        format!("db{}", COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn url(&self, uri: &str) -> String {
        format!("{}:{}/api/v1{uri}", Self::url_base(), self.port)
    }

    fn url_base() -> String {
        format!("{PROTOCOL}://{HOST}")
    }
}

impl Drop for TestServerImpl {
    fn drop(&mut self) {
        Self::shutdown_server(self).unwrap();
        Self::remove_dir_if_exists(&self.dir).unwrap();
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let mutex = MUTEX.get().unwrap();
        let _guard = loop {
            if let Ok(g) = mutex.try_lock() {
                break g;
            }
        };
        let global_server = SERVER.get().unwrap();
        let mut server_guard = global_server.try_write().unwrap();
        let server = server_guard.as_mut().unwrap();

        if server.instances == 1 {
            *server_guard = None;
        } else {
            server.instances -= 1;
        }
    }
}
