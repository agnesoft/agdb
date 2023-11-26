use assert_cmd::prelude::*;
use reqwest::Client;
use std::collections::HashMap;
use std::panic::Location;
use std::path::Path;
use std::process::Child;
use std::process::Command;
use std::sync::atomic::AtomicU16;

const CONFIG_FILE: &str = "agdb_server.yaml";
const HOST: &str = "http://127.0.0.1";
const DEFAULT_PORT: u16 = 3000;
static PORT: AtomicU16 = AtomicU16::new(DEFAULT_PORT);

pub struct TestServer {
    dir: String,
    port: u16,
    process: Child,
    client: Client,
}

impl TestServer {
    #[track_caller]
    pub fn new() -> anyhow::Result<Self> {
        let caller = Location::caller();
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

        let port = PORT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Self::remove_dir_if_exists(&dir);
        std::fs::create_dir(&dir)?;

        if port != DEFAULT_PORT {
            std::fs::write(Path::new(&dir).join(CONFIG_FILE), format!("port: {}", port)).unwrap();
        }

        let process = Command::cargo_bin("agdb_server")?
            .current_dir(&dir)
            .spawn()?;

        let client = reqwest::Client::new();

        Ok(Self {
            dir,
            port,
            process,
            client,
        })
    }

    pub async fn get(&self, uri: &str) -> anyhow::Result<u16> {
        Ok(self
            .client
            .get(format!("{HOST}:{}{uri}", self.port))
            .send()
            .await?
            .status()
            .as_u16())
    }

    pub async fn post(&self, uri: &str, json: &HashMap<&str, &str>) -> anyhow::Result<u16> {
        Ok(self
            .client
            .post(format!("{HOST}:{}{uri}", self.port))
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
            .post(format!("{HOST}:{}{uri}", self.port))
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
        let response = self
            .client
            .post(format!("{HOST}:{}{uri}", self.port))
            .json(&json)
            .send()
            .await?;
        let status = response.status().as_u16();
        let response_content = String::from_utf8(response.bytes().await?.to_vec())?;

        Ok((status, response_content))
    }

    fn remove_dir_if_exists(dir: &str) {
        if Path::new(dir).exists() {
            std::fs::remove_dir_all(dir).unwrap();
        }
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let port = self.port;
        std::thread::spawn(move || {
            assert_eq!(
                reqwest::blocking::get(format!("{HOST}:{port}/shutdown"))
                    .unwrap()
                    .status()
                    .as_u16(),
                200
            );
        })
        .join()
        .unwrap();

        assert!(self.process.wait().unwrap().success());
        Self::remove_dir_if_exists(&self.dir);
    }
}
