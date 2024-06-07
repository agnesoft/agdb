use crate::wait_for_ready;
use crate::TestServer;
use crate::TestServerImpl;
use crate::ADMIN;
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
    let status = client
        .get(server.full_url("/test_error"))
        .send()
        .await?
        .status();
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    Ok(())
}

#[tokio::test]
async fn missing() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let client = reqwest::Client::new();
    let status = client
        .get(server.full_url("/missing"))
        .send()
        .await?
        .status();
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
        .post(server.full_url("/admin/shutdown"))
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
        .get(server.full_url("/openapi.json"))
        .send()
        .await?
        .status();
    assert_eq!(status, 200);
    Ok(())
}

#[tokio::test]
async fn config_reuse() -> anyhow::Result<()> {
    let mut server = TestServerImpl::new().await?;
    let mut client = AgdbApi::new(ReqwestClient::new(), &server.address);
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_shutdown().await?;
    assert!(server.process.wait()?.success());
    server.process = Command::cargo_bin("agdb_server")?
        .current_dir(&server.dir)
        .spawn()?;
    wait_for_ready(&client).await?;
    Ok(())
}

#[tokio::test]
async fn db_list_after_shutdown() -> anyhow::Result<()> {
    let mut server = TestServerImpl::new().await?;
    let mut client = AgdbApi::new(ReqwestClient::new(), &server.address);

    {
        client.user_login(ADMIN, ADMIN).await?;
        client.admin_user_add("userx", "userxpassword").await?;
        client.user_logout().await?;
        client.user_login("userx", "userxpassword").await?;
        client
            .db_add("userx", "mydb", agdb_api::DbType::Mapped)
            .await?;
        client.user_logout().await?;
        client.user_login(ADMIN, ADMIN).await?;
        client.admin_shutdown().await?;
        assert!(server.process.wait()?.success());
    }

    server.process = Command::cargo_bin("agdb_server")?
        .current_dir(&server.dir)
        .spawn()?;
    wait_for_ready(&client).await?;
    client.user_login("userx", "userxpassword").await?;
    let dbs = client.db_list().await?.1;
    assert_eq!(dbs.len(), 1);

    Ok(())
}

#[tokio::test]
async fn db_list_after_shutdown_corrupted_data() -> anyhow::Result<()> {
    let mut server = TestServerImpl::new().await?;
    let mut client = AgdbApi::new(ReqwestClient::new(), &server.address);

    {
        client.user_login(ADMIN, ADMIN).await?;
        client.admin_user_add("userx", "userxpassword").await?;
        client.user_logout().await?;
        client.user_login("userx", "userxpassword").await?;
        client
            .db_add("userx", "mydb", agdb_api::DbType::Mapped)
            .await?;
        client.user_logout().await?;
        client.user_login(ADMIN, ADMIN).await?;
        client.admin_shutdown().await?;
        assert!(server.process.wait()?.success());
    }

    std::fs::remove_dir_all(&server.data_dir)?;

    server.process = Command::cargo_bin("agdb_server")?
        .current_dir(&server.dir)
        .spawn()?;
    wait_for_ready(&client).await?;
    client.user_login("userx", "userxpassword").await?;
    let dbs = client.db_list().await?.1;
    assert_eq!(dbs.len(), 1);

    Ok(())
}

#[tokio::test]
async fn basepath_test() -> anyhow::Result<()> {
    let mut config = HashMap::<&str, serde_yaml::Value>::new();
    config.insert("admin", ADMIN.into());
    config.insert("data_dir", SERVER_DATA_DIR.into());
    config.insert("basepath", "/public".into());
    config.insert("cluster", Vec::<String>::new().into());

    let _server = TestServerImpl::with_config(config).await?;

    // If the base path does not work the server
    // will not ever be considered ready, see
    // TestServer implementation for details.

    Ok(())
}
