use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb_api::LogLevelFilter;

async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .admin_set_log_level(LogLevelFilter::Debug)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_set_log_level(LogLevelFilter::Off)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    let status = server
        .api
        .admin_set_log_level(LogLevelFilter::Error)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    let status = server
        .api
        .admin_set_log_level(LogLevelFilter::Warn)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    let status = server
        .api
        .admin_set_log_level(LogLevelFilter::Info)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    let status = server
        .api
        .admin_set_log_level(LogLevelFilter::Debug)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    let status = server
        .api
        .admin_set_log_level(LogLevelFilter::Trace)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![non_admin, no_token,];
