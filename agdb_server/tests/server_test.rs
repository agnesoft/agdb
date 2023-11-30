pub mod framework;

use crate::framework::TestServer;
use assert_cmd::cargo::CommandCargoExt;
use std::process::Command;

#[tokio::test]
async fn db_reuse_and_error() -> anyhow::Result<()> {
    let mut server = TestServer::new()?;
    assert_eq!(server.get("/test_error").await?, 500);
    assert_eq!(server.get("/missing").await?, 404);
    assert_eq!(server.get("/admin/shutdown").await?, 200);
    assert!(server.process.wait().unwrap().success());
    server.process = Command::cargo_bin("agdb_server")?
        .current_dir(&server.dir)
        .spawn()?;
    Ok(())
}

#[tokio::test]
async fn openapi() -> anyhow::Result<()> {
    let server = TestServer::new()?;
    assert_eq!(server.get("").await?, 200);
    assert_eq!(server.get("/openapi.json").await?, 200);
    Ok(())
}
