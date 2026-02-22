use crate::ADMIN;
use crate::TestServer;
use crate::TestServerImpl;
use crate::next_user_name;
use crate::reqwest_client;
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;

#[tokio::test]
async fn set_log_level() -> anyhow::Result<()> {
    let server = TestServerImpl::new().await?;
    let mut client = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &server.address,
    );
    client.user_login(ADMIN, ADMIN).await?;

    let (_, admin_status) = client.admin_status().await?;
    assert_eq!(admin_status.log_level, agdb_api::LogLevelFilter::Info);

    client
        .admin_set_log_level(agdb_api::LogLevelFilter::Debug)
        .await?;

    let (_, admin_status) = client.admin_status().await?;
    assert_eq!(admin_status.log_level, agdb_api::LogLevelFilter::Debug);

    client
        .admin_set_log_level(agdb_api::LogLevelFilter::Debug)
        .await?;

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
