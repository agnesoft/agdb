mod test_config;

use crate::test_config::TestConfig;
use assert_cmd::prelude::*;
use std::collections::HashMap;
use std::process::Command;

#[tokio::test]
async fn config_port() -> anyhow::Result<()> {
    let test_config = TestConfig::new_content("port: 4000");
    let mut server = Command::cargo_bin("agdb_server")?
        .current_dir(&test_config.dir)
        .spawn()?;
    assert_eq!(
        reqwest::get("http://127.0.0.1:4000/error").await?.status(),
        500
    );
    assert!(reqwest::get("http://127.0.0.1:4000/shutdown")
        .await?
        .status()
        .is_success());
    assert!(server.wait()?.success());
    Ok(())
}

#[tokio::test]
async fn openapi() -> anyhow::Result<()> {
    let test_config = TestConfig::new_content("port: 5000");
    let mut server = Command::cargo_bin("agdb_server")?
        .current_dir(&test_config.dir)
        .spawn()?;
    assert!(reqwest::get("http://127.0.0.1:5000/openapi")
        .await?
        .status()
        .is_success());
    assert!(reqwest::get("http://127.0.0.1:5000/shutdown")
        .await?
        .status()
        .is_success());
    assert!(server.wait()?.success());
    Ok(())
}

#[tokio::test]
async fn create_user() -> anyhow::Result<()> {
    let test_config = TestConfig::new();
    let mut server = Command::cargo_bin("agdb_server")?
        .current_dir(&test_config.dir)
        .spawn()?;
    let client = reqwest::Client::new();

    let mut user = HashMap::new();
    user.insert("name", "a");
    user.insert("password", "");

    assert_eq!(
        client
            .post("http://127.0.0.1:3000/create_user")
            .json(&user)
            .send()
            .await?
            .status()
            .as_u16(),
        461
    );

    user.insert("name", "alice");

    assert_eq!(
        client
            .post("http://127.0.0.1:3000/create_user")
            .json(&user)
            .send()
            .await?
            .status()
            .as_u16(),
        462
    );

    user.insert("password", "mypassword123");

    assert_eq!(
        client
            .post("http://127.0.0.1:3000/create_user")
            .json(&user)
            .send()
            .await?
            .status()
            .as_u16(),
        201
    );

    assert_eq!(
        client
            .post("http://127.0.0.1:3000/create_user")
            .json(&user)
            .send()
            .await?
            .status()
            .as_u16(),
        463
    );

    assert!(client
        .get("http://127.0.0.1:3000/shutdown")
        .send()
        .await?
        .status()
        .is_success());
    assert!(server.wait()?.success());
    Ok(())
}
