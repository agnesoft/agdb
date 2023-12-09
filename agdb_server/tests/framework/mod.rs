use anyhow::anyhow;
use assert_cmd::prelude::*;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Child;
use std::process::Command;
use std::sync::atomic::AtomicU16;
use std::time::Duration;

pub const USER_CHANGE_PASSWORD_URI: &str = "/user/change_password";
pub const ADMIN_USER_CREATE_URI: &str = "/admin/user/create";
pub const DB_ADD_URI: &str = "/db/add";
pub const DB_USER_ADD_URI: &str = "/db/user/add";
pub const DB_DELETE_URI: &str = "/db/delete";
pub const DB_LIST_URI: &str = "/db/list";
pub const ADMIN_DB_LIST_URI: &str = "/admin/db/list";
pub const ADMIN_USER_LIST_URI: &str = "/admin/user/list";
pub const ADMIN_CHANGE_PASSWORD_URI: &str = "/admin/user/change_password";
pub const DB_REMOVE_URI: &str = "/db/remove";
pub const USER_LOGIN_URI: &str = "/user/login";
pub const SHUTDOWN_URI: &str = "/admin/shutdown";
pub const STATUS_URI: &str = "/status";

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Db {
    pub name: String,
    pub db_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct User<'a> {
    pub name: &'a str,
    pub password: &'a str,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserStatus {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangePassword<'a> {
    pub name: &'a str,
    pub password: &'a str,
    pub new_password: &'a str,
}

pub struct TestServer {
    pub dir: String,
    pub port: u16,
    pub process: Child,
    pub client: Client,
    pub admin: String,
    pub admin_password: String,
    pub admin_token: Option<String>,
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
            admin_token: None,
        };

        let mut error = anyhow!("Failed to start server");

        for _ in 0..RETRY_ATTEMPS {
            match server
                .client
                .get(format!("{}:{}/api/v1{STATUS_URI}", Self::url_base(), port))
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

    pub async fn init_admin(&mut self) -> anyhow::Result<Option<String>> {
        let admin = User {
            name: &self.admin,
            password: &self.admin_password,
        };
        let response = self.post(USER_LOGIN_URI, &admin, &None).await?;
        assert_eq!(response.0, 200);
        self.admin_token = Some(response.1);
        Ok(self.admin_token.clone())
    }

    pub async fn init_user(
        &mut self,
        name: &str,
        password: &str,
    ) -> anyhow::Result<Option<String>> {
        let user = User { name, password };
        if self.admin_token.is_none() {
            self.init_admin().await?;
        }
        assert_eq!(
            self.post(ADMIN_USER_CREATE_URI, &user, &self.admin_token.clone())
                .await?
                .0,
            201
        );
        let response = self.post(USER_LOGIN_URI, &user, &None).await?;
        assert_eq!(response.0, 200);
        Ok(Some(response.1))
    }

    pub async fn init_db(
        &self,
        name: &str,
        db_type: &str,
        user_token: &Option<String>,
    ) -> anyhow::Result<()> {
        let db = Db {
            name: name.to_string(),
            db_type: db_type.to_string(),
        };
        let (status, _) = self.post(DB_ADD_URI, &db, user_token).await?;
        assert_eq!(status, 201);
        Ok(())
    }

    pub async fn post<T: Serialize>(
        &self,
        uri: &str,
        json: &T,
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
            let token = self.admin_token.clone();

            std::thread::spawn(move || -> anyhow::Result<()> {
                let admin_token = if let Some(t) = token {
                    t
                } else {
                    reqwest::blocking::Client::new()
                        .post(format!(
                            "{}:{}/api/v1{USER_LOGIN_URI}",
                            Self::url_base(),
                            port
                        ))
                        .json(&admin)
                        .send()?
                        .text()?
                };

                assert_eq!(
                    reqwest::blocking::Client::new()
                        .get(format!(
                            "{}:{}/api/v1{SHUTDOWN_URI}",
                            Self::url_base(),
                            port
                        ))
                        .bearer_auth(admin_token)
                        .send()?
                        .status()
                        .as_u16(),
                    204
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
