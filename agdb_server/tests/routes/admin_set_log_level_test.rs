use crate::ADMIN;
use crate::TestServer;
use crate::next_user_name;

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

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .admin_set_log_level(agdb_api::LogLevelFilter::Debug)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_set_log_level(agdb_api::LogLevelFilter::Debug)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}
