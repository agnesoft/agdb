use assert_cmd::prelude::*;
use reqwest::Client;
use std::collections::HashMap;
use std::panic::Location;
use std::path::Path;
use std::process::Child;
use std::process::Command;
use std::sync::atomic::AtomicU16;

const BINARY: &str = "agdb_server";
const CONFIG_FILE: &str = "agdb_server.yaml";
const HOST_IP: &str = "127.0.0.1";
const HOST: &str = "http://127.0.0.1";
const DEFAULT_PORT: u16 = 3000;
const ADMIN: &str = "admin";
static PORT: AtomicU16 = AtomicU16::new(DEFAULT_PORT);

pub struct TestServer {
    pub dir: String,
    pub port: u16,
    pub process: Child,
    pub client: Client,
    pub admin: String,
    pub admin_password: String,
    pub admin_token: String,
}

impl TestServer {
    pub async fn new(port_offset: u16, caller: &Location<'static>) -> anyhow::Result<Self> {
        let dir = format!(
            "{}.{}.{}.test",
            Path::new(caller.file())
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            caller.line(),
            caller.column()
        );

        let port = port_offset + PORT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Self::remove_dir_if_exists(&dir);
        std::fs::create_dir(&dir)?;

        println!("We have a dir...");

        if port != DEFAULT_PORT {
            let mut config = HashMap::<&str, serde_yaml::Value>::new();
            config.insert("host", HOST_IP.into());
            config.insert("port", port.into());
            config.insert("admin", ADMIN.into());

            let file = std::fs::File::options()
                .create_new(true)
                .write(true)
                .open(Path::new(&dir).join(CONFIG_FILE))?;
            serde_yaml::to_writer(file, &config)?;

            println!("We have a config...");
        }

        let process = Command::cargo_bin(BINARY)?.current_dir(&dir).spawn()?;

        println!("We have a running server...");

        let client = reqwest::Client::new();

        let mut server = Self {
            dir,
            port,
            process,
            client,
            admin: ADMIN.to_string(),
            admin_password: ADMIN.to_string(),
            admin_token: String::new(),
        };

        let mut admin = HashMap::<&str, &str>::new();
        admin.insert("name", &server.admin);
        admin.insert("password", &server.admin_password);

        server.admin_token = server.post_response("/user/login", &admin).await?.1;

        println!("We have an admin token: {}...", server.admin_token);

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

    fn remove_dir_if_exists(dir: &str) {
        if Path::new(dir).exists() {
            std::fs::remove_dir_all(dir).unwrap();
        }
    }

    fn url(&self, uri: &str) -> String {
        format!("{HOST}:{}/api/v1{uri}", self.port)
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let port = self.port;
        let admin_token = self.admin_token.clone();

        if self.process.try_wait().unwrap().is_none() {
            std::thread::spawn(move || {
                assert_eq!(
                    reqwest::blocking::Client::new()
                        .get(format!("{HOST}:{}/api/v1/admin/shutdown", port))
                        .bearer_auth(admin_token)
                        .send()
                        .unwrap()
                        .status()
                        .as_u16(),
                    200
                );
            })
            .join()
            .unwrap();

            assert!(self.process.wait().unwrap().success());
        }

        Self::remove_dir_if_exists(&self.dir);
    }
}
