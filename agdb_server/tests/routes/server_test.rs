use crate::TestServer;
use crate::ADMIN;
use assert_cmd::cargo::CommandCargoExt;
use reqwest::StatusCode;
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
        .get(server.url("/admin/shutdown"))
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
    let (status, spec) = server.api.openapi().await?;
    assert_eq!(status, 200);
    assert!(!spec.is_empty());
    Ok(())
}

#[tokio::test]
async fn db_config_reuse() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_shutdown().await?;

    assert!(server.process.wait()?.success());
    server.process = Command::cargo_bin("agdb_server")?
        .current_dir(&server.dir)
        .spawn()?;
    Ok(())
}
