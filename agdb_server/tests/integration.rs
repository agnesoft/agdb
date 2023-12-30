mod routes;

use anyhow::anyhow;
use assert_cmd::prelude::*;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Child;
use std::process::Command;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering;
use std::time::Duration;

pub const DB_LIST_URI: &str = "/db/list";
pub const ADMIN_DB_LIST_URI: &str = "/admin/db/list";
pub const ADMIN_USER_LIST_URI: &str = "/admin/user/list";
pub const SHUTDOWN_URI: &str = "/admin/shutdown";
pub const STATUS_URI: &str = "/status";

pub const NO_TOKEN: &Option<String> = &None;

const BINARY: &str = "agdb_server";
const CONFIG_FILE: &str = "agdb_server.yaml";
const SERVER_DATA_DIR: &str = "agdb_server_data";
const PROTOCOL: &str = "http";
const HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 3000;
const ADMIN: &str = "admin";
const RETRY_TIMEOUT: Duration = Duration::from_secs(1);
const RETRY_ATTEMPS: u16 = 3;
const SHUTDOWN_RETRY_TIMEOUT: Duration = Duration::from_millis(100);
const SHUTDOWN_RETRY_ATTEMPTS: u16 = 100;

static PORT: AtomicU16 = AtomicU16::new(DEFAULT_PORT);
static COUNTER: AtomicU16 = AtomicU16::new(1);

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct DbUser {
    user: String,
    role: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Db {
    pub name: String,
    pub db_type: String,
    pub role: String,
    pub size: u64,
    pub backup: u64,
}

#[derive(Serialize, Deserialize)]
pub struct UserCredentials<'a> {
    pub password: &'a str,
}

#[derive(Serialize, Deserialize)]
pub struct UserLogin<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserStatus {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangePassword<'a> {
    pub password: &'a str,
    pub new_password: &'a str,
}

pub struct TestServer {
    pub dir: String,
    pub data_dir: String,
    pub client: reqwest::Client,
    pub port: u16,
    pub process: Child,
    pub admin: String,
    pub admin_password: String,
    pub admin_token: Option<String>,
}

pub struct ServerUser {
    pub name: String,
    pub token: Option<String>,
}

impl TestServer {
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
        let client = reqwest::Client::new();

        let mut error = anyhow!("Failed to start server");

        for _ in 0..RETRY_ATTEMPS {
            match client
                .get(format!("{}:{}/api/v1{STATUS_URI}", Self::url_base(), port))
                .send()
                .await
            {
                Ok(_) => {
                    let credentials = UserLogin {
                        username: ADMIN,
                        password: ADMIN,
                    };
                    let response = client
                        .post(format!("{}:{}/api/v1/user/login", Self::url_base(), port))
                        .json(&credentials)
                        .send()
                        .await?;
                    let admin_token = Some(response.text().await?);
                    let server = Self {
                        dir,
                        data_dir,
                        client,
                        port,
                        process,
                        admin: ADMIN.to_string(),
                        admin_password: ADMIN.to_string(),
                        admin_token,
                    };
                    return Ok(server);
                }
                Err(e) => {
                    error = e.into();
                }
            }
            std::thread::sleep(RETRY_TIMEOUT);
        }

        Err(error)
    }

    pub async fn delete(&self, uri: &str, token: &Option<String>) -> anyhow::Result<u16> {
        let mut request = self.client.delete(self.url(uri));

        if let Some(token) = token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;
        let status = response.status().as_u16();

        Ok(status)
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        uri: &str,
        token: &Option<String>,
    ) -> anyhow::Result<(u16, anyhow::Result<T>)> {
        let mut request = self.client.get(self.url(uri));

        if let Some(token) = token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;
        let status = response.status().as_u16();

        Ok((status, response.json().await.map_err(|e| anyhow!(e))))
    }

    pub async fn init_user(&self) -> anyhow::Result<ServerUser> {
        let name = format!("db_user{}", COUNTER.fetch_add(1, Ordering::Relaxed));
        let credentials = Some(UserCredentials { password: &name });
        assert_eq!(
            self.post(
                &format!("/admin/user/{name}/add"),
                &credentials,
                &self.admin_token
            )
            .await?
            .0,
            201
        );
        let response = self
            .post(
                "/user/login",
                &Some(UserLogin {
                    username: &name,
                    password: &name,
                }),
                &None,
            )
            .await?;
        assert_eq!(response.0, 200);
        Ok(ServerUser {
            name,
            token: Some(response.1),
        })
    }

    pub async fn init_db(&self, db_type: &str, server_user: &ServerUser) -> anyhow::Result<String> {
        let name = format!(
            "{}/db{}",
            server_user.name,
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let uri = format!("/db/{name}/add?db_type={db_type}",);
        let status = self.post::<()>(&uri, &None, &server_user.token).await?.0;
        assert_eq!(status, 201);
        Ok(name)
    }

    pub async fn post<T: Serialize>(
        &self,
        uri: &str,
        json: &Option<T>,
        token: &Option<String>,
    ) -> anyhow::Result<(u16, String)> {
        let mut request = self.client.post(self.url(uri));

        if let Some(json) = json {
            request = request.json(json);
        }

        if let Some(token) = token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        Ok((response.status().as_u16(), response.text().await?))
    }

    pub async fn put<T: Serialize>(
        &self,
        uri: &str,
        json: &Option<T>,
        token: &Option<String>,
    ) -> anyhow::Result<u16> {
        let mut request = self.client.put(self.url(uri));

        if let Some(json) = json {
            request = request.json(json);
        }

        if let Some(token) = token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        Ok(response.status().as_u16())
    }

    fn remove_dir_if_exists(dir: &str) -> anyhow::Result<()> {
        if Path::new(dir).exists() {
            std::fs::remove_dir_all(dir)?;
        }

        Ok(())
    }

    fn shutdown_server(&mut self) -> anyhow::Result<()> {
        if self.process.try_wait()?.is_some() {
            return Ok(());
        }

        let port = self.port;
        let mut admin = HashMap::<&str, String>::new();
        admin.insert("name", self.admin.clone());
        admin.insert("password", self.admin_password.clone());
        let admin_token = self.admin_token.clone().unwrap_or_default();

        std::thread::spawn(move || -> anyhow::Result<()> {
            assert_eq!(
                reqwest::blocking::Client::new()
                    .post(format!(
                        "{}:{}/api/v1{SHUTDOWN_URI}",
                        Self::url_base(),
                        port
                    ))
                    .bearer_auth(admin_token)
                    .send()?
                    .status()
                    .as_u16(),
                202
            );

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

    fn url(&self, uri: &str) -> String {
        format!("{}:{}/api/v1{uri}", Self::url_base(), self.port)
    }

    fn url_base() -> String {
        format!("{PROTOCOL}://{HOST}")
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        Self::shutdown_server(self).unwrap();
        Self::remove_dir_if_exists(&self.dir).unwrap();
    }
}
