use assert_cmd::prelude::*;
use std::process::Command;

#[tokio::test]
async fn endpoints() -> anyhow::Result<()> {
    let mut server = Command::cargo_bin("agdb_server")?.spawn()?;
    assert_eq!(
        reqwest::get("http://127.0.0.1:3000")
            .await?
            .status()
            .as_u16(),
        200
    );
    assert_eq!(
        reqwest::get("http://127.0.0.1:3000/missing")
            .await?
            .status()
            .as_u16(),
        404
    );
    assert_eq!(
        reqwest::get("http://127.0.0.1:3000/error")
            .await?
            .status()
            .as_u16(),
        500
    );
    assert!(reqwest::get("http://127.0.0.1:3000/shutdown")
        .await?
        .status()
        .is_success());
    assert!(server.wait()?.success());
    Ok(())
}
