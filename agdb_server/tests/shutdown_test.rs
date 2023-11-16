use assert_cmd::prelude::*;
use std::process::Command;

#[tokio::test]
async fn shutdown() -> anyhow::Result<()> {
    let mut server = Command::cargo_bin("agdb_server")?.spawn()?;
    assert!(reqwest::get("http://127.0.0.1:3000/shutdown")
        .await?
        .status()
        .is_success());
    assert!(server.wait()?.success());
    Ok(())
}
