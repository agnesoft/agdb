use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn add() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server.api.admin_user_add(user, user).await?;
    assert_eq!(status, 201);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn add_existing() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    let status = server
        .api
        .admin_user_add(user, user)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 463);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn name_too_short() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_user_add("a", "password123")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 462);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn password_too_short() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_user_add("user123", "pswd")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 461);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .admin_user_add(user, user)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_user_add("user", "password123")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __add_type_def(),
        __add_existing_type_def(),
        __name_too_short_type_def(),
        __password_too_short_type_def(),
        __non_admin_type_def(),
        __no_token_type_def(),
    ]
}
