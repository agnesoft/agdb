use crate::TestServer;
use crate::TestServerImpl;
use crate::ADMIN;
use crate::HOST;
use crate::SERVER_DATA_DIR;
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;
use assert_cmd::cargo::CommandCargoExt;
use reqwest::StatusCode;
use std::collections::HashMap;
use std::process::Command;

#[tokio::test]
async fn error() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let client = reqwest::Client::new();
    let status = client.get(server.url("/test_error")).send().await?.status();
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    Ok(())
}

#[tokio::test]
async fn missing() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let client = reqwest::Client::new();
    let status = client.get(server.url("/missing")).send().await?.status();
    assert_eq!(status, StatusCode::NOT_FOUND);
    Ok(())
}

#[tokio::test]
async fn status() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server.api.status().await?;
    assert_eq!(status, 200);
    Ok(())
}

#[tokio::test]
async fn shutdown_no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server.api.admin_shutdown().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn shutdown_bad_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let client = reqwest::Client::new();
    let status = client
        .post(server.url("/admin/shutdown"))
        .bearer_auth("bad")
        .send()
        .await?
        .status();
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    Ok(())
}

#[tokio::test]
async fn openapi() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let client = reqwest::Client::new();
    let status = client
        .get(server.url("/openapi.json"))
        .send()
        .await?
        .status();
    assert_eq!(status, 200);
    Ok(())
}

#[tokio::test]
async fn db_config_reuse() -> anyhow::Result<()> {
    let mut server = TestServerImpl::new().await?;
    let mut client = AgdbApi::new(ReqwestClient::new(), &TestServer::url_base(), server.port);
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_shutdown().await?;
    assert!(server.process.wait()?.success());
    server.process = Command::cargo_bin("agdb_server")?
        .current_dir(&server.dir)
        .spawn()?;
    Ok(())
}

#[tokio::test]
async fn db_cluster() -> anyhow::Result<()> {
    let port1 = TestServerImpl::next_port();
    let port2 = TestServerImpl::next_port();
    let port3 = TestServerImpl::next_port();
    let cluster = vec![
        format!("{HOST}:{port1}"),
        format!("{HOST}:{port2}"),
        format!("{HOST}:{port3}"),
    ];

    let mut config1 = HashMap::<&str, serde_yaml::Value>::new();
    config1.insert("host", HOST.into());
    config1.insert("port", port1.into());
    config1.insert("admin", ADMIN.into());
    config1.insert("data_dir", SERVER_DATA_DIR.into());
    config1.insert("cluster", cluster.into());

    let mut config2 = config1.clone();
    config2.insert("port", port2.into());

    let mut config3 = config1.clone();
    config3.insert("port", port3.into());

    let _server1 = TestServerImpl::with_config(&config1).await?;
    let _server2 = TestServerImpl::with_config(&config2).await?;
    let _server3 = TestServerImpl::with_config(&config3).await?;

    Ok(())
}
