use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServerImpl;
use crate::test_server::reqwest_client;
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;

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

pub(super) const TESTS: &[Test] = tests![set_log_level,];
