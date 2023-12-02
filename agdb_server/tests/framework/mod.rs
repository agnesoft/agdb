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
const TIMEOUT: Duration = Duration::from_secs(3);
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

        server
            .client
            .get(format!("{}:{}/api/v1/status", Self::url_base(), port))
            .timeout(TIMEOUT)
            .send()
            .await?;

        Ok(server)
    }

    pub async fn get(&self, uri: &str) -> anyhow::Result<u16> {
        Ok(self
            .client
            .get(self.url(uri))
            .send()
            .await?
            .status()
            .as_u16())
    }

    pub async fn get_auth(&self, uri: &str, token: &str) -> anyhow::Result<u16> {
        Ok(self
            .client
            .get(self.url(uri))
            .bearer_auth(token)
            .send()
            .await?
            .status()
            .as_u16())
    }

    pub async fn get_auth_response(&self, uri: &str, token: &str) -> anyhow::Result<(u16, String)> {
        let response = self
            .client
            .get(self.url(uri))
            .bearer_auth(token)
            .send()
            .await?;
        let status = response.status().as_u16();
        let response_content = response.text().await?;

        Ok((status, response_content))
    }

    pub async fn post(&self, uri: &str, json: &HashMap<&str, &str>) -> anyhow::Result<u16> {
        Ok(self
            .client
            .post(self.url(uri))
            .json(&json)
            .send()
            .await?
            .status()
            .as_u16())
    }

    pub async fn post_auth(
        &self,
        uri: &str,
        token: &str,
        json: &HashMap<&str, &str>,
    ) -> anyhow::Result<u16> {
        Ok(self
            .client
            .post(self.url(uri))
            .bearer_auth(token)
            .json(&json)
            .send()
            .await?
            .status()
            .as_u16())
    }

    pub async fn post_response(
        &self,
        uri: &str,
        json: &HashMap<&str, &str>,
    ) -> anyhow::Result<(u16, String)> {
        let response = self.client.post(self.url(uri)).json(&json).send().await?;
        let status = response.status().as_u16();
        let response_content = response.text().await?;

        Ok((status, response_content))
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
