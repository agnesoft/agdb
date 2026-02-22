use crate::ADMIN;
use crate::TestServer;

#[tokio::test]
async fn set_log_level() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;

    let (_, admin_status) = server.api.admin_status().await?;
    assert_eq!(admin_status.log_level, agdb_api::LogLevelFilter::Info);

    server
        .api
        .admin_set_log_level(agdb_api::LogLevelFilter::Debug)
        .await?;

    let (_, admin_status) = server.api.admin_status().await?;
    assert_eq!(admin_status.log_level, agdb_api::LogLevelFilter::Debug);

    Ok(())
}
