use anyhow::anyhow;
use assert_cmd::prelude::*;
use reqwest::Client;
use std::collections::HashMap;
use std::path::Path;
use std::process::Child;
use std::process::Command;
use std::sync::atomic::AtomicU16;
use std::time::Duration;

const BINARY: &str = "agdb_server";
const CONFIG_FILE: &str = "agdb_server.yaml";
const PROTOCOL: &str = "http";
const HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 3000;
const ADMIN: &str = "admin";
const RETRY_TIMEOUT: Duration = Duration::from_secs(1);
const RETRY_ATTEMPS: u16 = 3;
pub const NO_TOKEN: &Option<String> = &None;
static PORT: AtomicU16 = AtomicU16::new(DEFAULT_PORT);

pub struct TestServer {
    pub dir: String,
    pub port: u16,
    pub process: Child,
    pub client: Client,
    pub admin: String,
    pub admin_password: String,
}

impl TestServer {
    pub async fn new() -> anyhow::Result<Self> {
        let port = PORT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = format!("{BINARY}.{port}.test");

        Self::remove_dir_if_exists(&dir)?;
        std::fs::create_dir(&dir)?;

        if port != DEFAULT_PORT {
            let mut config = HashMap::<&str, serde_yaml::Value>::new();
            config.insert("host", HOST.into());
            config.insert("port", port.into());
            config.insert("admin", ADMIN.into());

            let file = std::fs::File::options()
                .create_new(true)
                .write(true)
                .open(Path::new(&dir).join(CONFIG_FILE))?;
            serde_yaml::to_writer(file, &config)?;
        }

        let process = Command::cargo_bin(BINARY)?.current_dir(&dir).spawn()?;
        let client = reqwest::Client::new();
        let server = Self {
            dir,
            port,
            process,
            client,
            admin: ADMIN.to_string(),
            admin_password: ADMIN.to_string(),
        };

        let mut error = anyhow!("Failed to start server");

        for _ in 0..RETRY_ATTEMPS {
            match server
                .client
                .get(format!("{}:{}/api/v1/status", Self::url_base(), port))
                .send()
                .await
            {
                Ok(_) => return Ok(server),
                Err(e) => {
                    error = e.into();
                }
            }
            std::thread::sleep(RETRY_TIMEOUT);
        }

        Err(error)
    }

    pub async fn get(&self, uri: &str, token: &Option<String>) -> anyhow::Result<(u16, String)> {
        let mut request = self.client.get(self.url(uri));

        if let Some(token) = token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        Ok((response.status().as_u16(), response.text().await?))
    }

    pub async fn init_admin(&self) -> anyhow::Result<Option<String>> {
        let mut admin = HashMap::<&str, &str>::new();
        admin.insert("name", &self.admin);
        admin.insert("password", &self.admin_password);
        let response = self.post("/user/login", &admin, &None).await?;
        assert_eq!(response.0, 200);
        Ok(Some(response.1))
    }

    pub async fn init_user(&self, name: &str, password: &str) -> anyhow::Result<Option<String>> {
        let mut user = HashMap::<&str, &str>::new();
        user.insert("name", name);
        user.insert("password", password);
        let admin_token = self.init_admin().await?;
        assert_eq!(
            self.post("/admin/user/create", &user, &admin_token)
                .await?
                .0,
            201
        );
        let response = self.post("/user/login", &user, &None).await?;
        assert_eq!(response.0, 200);
        Ok(Some(response.1))
    }

    pub async fn post(
        &self,
        uri: &str,
        json: &HashMap<&str, &str>,
        token: &Option<String>,
    ) -> anyhow::Result<(u16, String)> {
        let mut request = self.client.post(self.url(uri)).json(&json);

        if let Some(token) = token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        Ok((response.status().as_u16(), response.text().await?))
    }

    fn remove_dir_if_exists(dir: &str) -> anyhow::Result<()> {
        if Path::new(dir).exists() {
            std::fs::remove_dir_all(dir)?;
        }

        Ok(())
    }

    fn shutdown_server(&mut self) -> anyhow::Result<()> {
        if self.process.try_wait()?.is_none() {
            let port = self.port;
            let mut admin = HashMap::<&str, String>::new();
            admin.insert("name", self.admin.clone());
            admin.insert("password", self.admin_password.clone());

            std::thread::spawn(move || -> anyhow::Result<()> {
                let admin_token = reqwest::blocking::Client::new()
                    .post(format!("{}:{}/api/v1/user/login", Self::url_base(), port))
                    .json(&admin)
                    .send()?
                    .text()?;

                assert_eq!(
                    reqwest::blocking::Client::new()
                        .get(format!(
                            "{}:{}/api/v1/admin/shutdown",
                            Self::url_base(),
                            port
                        ))
                        .bearer_auth(admin_token)
                        .send()?
                        .status()
                        .as_u16(),
                    200
                );

                Ok(())
            })
            .join()
            .map_err(|e| anyhow!("{:?}", e))??;
        }

        assert!(self.process.wait()?.success());

        Ok(())
    }

    fn url(&self, uri: &str) -> String {
        format!("{}:{}/api/v1{uri}", Self::url_base(), self.port)
    }

    fn url_base() -> String {
        format!("{PROTOCOL}://{HOST}")
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let shutdown_result = Self::shutdown_server(self);

        if shutdown_result.is_err() {
            let _ = self.process.kill();
            let _ = self.process.wait();
        }

        shutdown_result.unwrap();
        Self::remove_dir_if_exists(&self.dir).unwrap();
    }
}
