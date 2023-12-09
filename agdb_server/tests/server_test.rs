pub mod framework;

use crate::framework::TestServer;
use crate::framework::NO_TOKEN;
use crate::framework::SHUTDOWN_URI;
use crate::framework::STATUS_URI;
use assert_cmd::cargo::CommandCargoExt;
use std::process::Command;

#[tokio::test]
async fn error() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(server.get::<()>("/test_error", NO_TOKEN).await?.0, 500);
    Ok(())
}

#[tokio::test]
async fn missing() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(server.get::<()>("/missing", NO_TOKEN).await?.0, 404);
    Ok(())
}

#[tokio::test]
async fn status() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(server.get::<()>(STATUS_URI, NO_TOKEN).await?.0, 200);
    Ok(())
}

#[tokio::test]
async fn shutdown_no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(server.get::<()>(SHUTDOWN_URI, NO_TOKEN).await?.0, 401);
    Ok(())
}

#[tokio::test]
async fn shutdown_bad_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let token = Some("bad".to_string());
    assert_eq!(server.get::<()>(SHUTDOWN_URI, &token).await?.0, 401);
    Ok(())
}

#[tokio::test]
async fn db_reuse() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token = server.init_admin().await?;
    assert_eq!(server.get::<()>(SHUTDOWN_URI, &token).await?.0, 204);
    assert!(server.process.wait()?.success());
    server.process = Command::cargo_bin("agdb_server")?
        .current_dir(&server.dir)
        .spawn()?;
    Ok(())
}

#[tokio::test]
async fn openapi() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(server.get::<()>("", NO_TOKEN).await?.0, 200);
    assert_eq!(
        server.get::<String>("/openapi.json", NO_TOKEN).await?.0,
        200
    );
    Ok(())
}
