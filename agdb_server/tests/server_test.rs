pub mod framework;

use crate::framework::TestServer;
use assert_cmd::cargo::CommandCargoExt;
use std::collections::HashMap;
use std::process::Command;

#[tokio::test]
async fn db_reuse_and_error() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    assert_eq!(server.get("/test_error").await?, 500);
    assert_eq!(server.get("/missing").await?, 404);
    assert_eq!(server.get("/admin/shutdown").await?, 401);
    assert_eq!(server.get_auth("/admin/shutdown", "bad_token").await?, 403);

    let mut admin = HashMap::<&str, &str>::new();
    admin.insert("name", &server.admin);
    admin.insert("password", &server.admin_password);
    let token = server.post_response("/user/login", &admin).await?.1;

    assert_eq!(server.get_auth("/admin/shutdown", &token).await?, 200);
    assert!(server.process.wait()?.success());

    server.process = Command::cargo_bin("agdb_server")?
        .current_dir(&server.dir)
        .spawn()?;

    Ok(())
}

#[tokio::test]
async fn openapi() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(server.get("").await?, 200);
    assert_eq!(server.get("/openapi.json").await?, 200);
    Ok(())
}
