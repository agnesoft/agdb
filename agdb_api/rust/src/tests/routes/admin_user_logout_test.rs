use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn logout() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let token = server.api.token.clone();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_logout(user).await?;
    server.api.token = token;
    let status = server.api.db_list().await.unwrap_err().status;
    assert_eq!(status, 401);

    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn unknown_user() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_user_logout("unknown_user")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);

    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server.api.admin_user_logout(user).await.unwrap_err().status;
    assert_eq!(status, 401);

    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_user_logout("user")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);

    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __logout_type_def(),
        __unknown_user_type_def(),
        __non_admin_type_def(),
        __no_token_type_def(),
    ]
}
