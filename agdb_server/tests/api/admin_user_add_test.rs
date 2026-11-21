use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

async fn add() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server.api.admin_user_add(user, user).await?;
    assert_eq!(status, 201);
    Ok(())
}

async fn add_existing() -> Result<(), TestError> {
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

async fn name_too_short() -> Result<(), TestError> {
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

async fn password_too_short() -> Result<(), TestError> {
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

async fn non_admin() -> Result<(), TestError> {
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

async fn no_token() -> Result<(), TestError> {
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

pub(super) const TESTS: &[Test] = tests![
    add,
    add_existing,
    name_too_short,
    password_too_short,
    non_admin,
    no_token,
];
