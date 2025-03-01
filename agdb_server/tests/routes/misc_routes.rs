use crate::reqwest_client;
use crate::wait_for_ready;
use crate::TestServer;
use crate::TestServerImpl;
use crate::ADMIN;
use crate::CONFIG_FILE;
use agdb::QueryBuilder;
use agdb_api::AgdbApi;
use agdb_api::DbType;
use agdb_api::ReqwestClient;
use reqwest::StatusCode;
use std::path::Path;

#[tokio::test]
async fn missing() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let client = reqwest_client();
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
    let client = reqwest_client();
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
    let client = reqwest_client();
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
    let mut client = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &server.address,
    );
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_shutdown().await?;
    server.wait().await?;
    server.restart()?;
    wait_for_ready(&client).await?;
    Ok(())
}

#[tokio::test]
async fn db_list_after_shutdown() -> anyhow::Result<()> {
    let mut server = TestServerImpl::new().await?;
    let mut client = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &server.address,
    );

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
        server.wait().await?;
    }

    server.restart()?;
    wait_for_ready(&client).await?;
    client.user_login("userx", "userxpassword").await?;
    let dbs = client.db_list().await?.1;
    assert_eq!(dbs.len(), 1);

    Ok(())
}

#[tokio::test]
async fn db_list_after_shutdown_corrupted_data() -> anyhow::Result<()> {
    let mut server = TestServerImpl::new().await?;
    let mut client = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &server.address,
    );

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
        server.wait().await?;
    }

    std::fs::remove_dir_all(Path::new(&server.data_dir).join("userx"))?;

    server.restart()?;
    wait_for_ready(&client).await?;
    client.user_login("userx", "userxpassword").await?;
    let dbs = client.db_list().await?.1;
    assert_eq!(dbs.len(), 1);

    Ok(())
}

#[cfg(feature = "studio")]
#[tokio::test]
async fn basepath_test() -> anyhow::Result<()> {
    let config = crate::config::ConfigImpl {
        bind: String::new(),
        address: String::new(),
        basepath: "/public".to_string(),
        admin: ADMIN.to_string(),
        log_level: tracing::level_filters::LevelFilter::INFO,
        data_dir: crate::SERVER_DATA_DIR.into(),
        pepper_path: String::new(),
        tls_certificate: String::new(),
        tls_key: String::new(),
        tls_root: String::new(),
        cluster_token: "test".to_string(),
        cluster_heartbeat_timeout_ms: 1000,
        cluster_term_timeout_ms: 3000,
        cluster: Vec::new(),
        cluster_node_id: 0,
        start_time: 0,
        pepper: None,
    };

    let server = TestServerImpl::with_config(config).await?;

    reqwest_client()
        .get(format!("{}/studio", server.address))
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

#[tokio::test]
async fn location_change_after_restart() -> anyhow::Result<()> {
    let mut server = TestServerImpl::new().await?;
    let mut client = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &server.address,
    );

    {
        client.user_login(ADMIN, ADMIN).await?;
        client.admin_user_add("user1", "userxpassword").await?;
        client.user_logout().await?;
        client.user_login("user1", "userxpassword").await?;
        client
            .db_add("user1", "mydb", agdb_api::DbType::Mapped)
            .await?;
        client
            .db_exec_mut(
                "user1",
                "mydb",
                &vec![QueryBuilder::insert().nodes().count(1).query().into()],
            )
            .await?;
        client.user_logout().await?;
        client.user_login(ADMIN, ADMIN).await?;
        client.admin_shutdown().await?;
        server.wait().await?;
    }

    server.restart()?;
    wait_for_ready(&client).await?;
    client.user_login("user1", "userxpassword").await?;
    let results = client
        .db_exec(
            "user1",
            "mydb",
            &vec![QueryBuilder::select().ids(1).query().into()],
        )
        .await?;

    assert_eq!(results.1.len(), 1);
    assert_eq!(results.1[0].result, 1);

    Ok(())
}

#[tokio::test]
async fn reset_admin_password() -> anyhow::Result<()> {
    let mut server = TestServerImpl::new().await?;
    let mut client = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &server.address,
    );

    {
        client.user_login(ADMIN, ADMIN).await?;
        client.admin_user_add("user1", "password123").await?;
        client.user_change_password(ADMIN, "lostpassword").await?;
        client.admin_shutdown().await?;
        server.wait().await?;
    }

    let config_file = Path::new(&server.dir).join(CONFIG_FILE);
    let new_config =
        std::fs::read_to_string(&config_file)?.replace("admin: admin", "admin: NEW_ADMIN");
    std::fs::write(config_file, new_config)?;

    server.restart()?;
    wait_for_ready(&client).await?;

    client.user_login("NEW_ADMIN", "NEW_ADMIN").await?;
    let list = client.admin_user_list().await;
    client.admin_shutdown().await?;
    server.wait().await?;
    assert_eq!(list?.1.len(), 3);

    Ok(())
}

#[tokio::test]
async fn memory_db_from_backup() -> anyhow::Result<()> {
    let mut server = TestServerImpl::new().await?;
    let mut client = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &server.address,
    );
    let owner = "user1";
    let db = "db1";

    {
        client.user_login(ADMIN, ADMIN).await?;
        client.admin_user_add(owner, "password123").await?;
        client.user_login(owner, "password123").await?;
        client.db_add(owner, db, DbType::Memory).await?;
        client
            .db_exec_mut(
                owner,
                db,
                &[QueryBuilder::insert().nodes().count(1).query().into()],
            )
            .await?;
        let status = client.db_backup(owner, db).await?;
        assert_eq!(status, 201);
        client.user_login(ADMIN, ADMIN).await?;
        client.admin_shutdown().await?;
        server.wait().await?;
    }

    server.restart()?;
    wait_for_ready(&client).await?;
    client.user_login(owner, "password123").await?;

    let result = client
        .db_exec(
            owner,
            db,
            &[QueryBuilder::select().node_count().query().into()],
        )
        .await?
        .1;
    assert_eq!(result[0].elements[0].values[0].value.to_u64()?, 1);

    Ok(())
}

#[cfg(feature = "studio")]
#[tokio::test]
async fn studio() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    reqwest_client()
        .get(server.url("/studio"))
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
