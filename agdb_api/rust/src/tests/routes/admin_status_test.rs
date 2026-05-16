use crate::DbKind;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn status() -> Result<(), TestError> {
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
    assert_eq!(admin_status.log_level, crate::LogLevelFilter::Info);

    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server.api.admin_status().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server.api.admin_status().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __status_type_def(),
        __non_admin_type_def(),
        __no_token_type_def(),
    ]
}
