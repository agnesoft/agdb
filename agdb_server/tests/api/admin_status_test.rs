use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb_api::DbKind;

async fn status() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    server
        .api
        .admin_db_add(ADMIN, "status_db", DbKind::Memory)
        .await?;

    let (status, admin_status) = server.api.admin_status().await?;
    assert_eq!(status, 200);
    assert_ne!(admin_status.dbs, 0);
    assert_ne!(admin_status.users, 0);
    assert_ne!(admin_status.logged_in_users, 0);
    assert_ne!(admin_status.size, 0);
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    assert_ne!(admin_status.memory, 0);
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    assert_eq!(admin_status.memory, 0);
    assert_eq!(admin_status.log_level, agdb_api::LogLevelFilter::Info);

    Ok(())
}

async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server.api.admin_status().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server.api.admin_status().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![status, non_admin, no_token,];
