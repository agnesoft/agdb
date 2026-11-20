use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

async fn change_password() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let status = server
        .api
        .admin_user_change_password(owner, "password123")
        .await?;
    assert_eq!(status, 201);
    let status = server.api.user_login(owner, "password123").await?;
    assert_eq!(status, 200);
    Ok(())
}

async fn password_short() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let status = server.api.admin_user_change_password(owner, "pswd").await?;
    assert_eq!(status, 201);
    Ok(())
}

async fn user_not_found() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_user_change_password("owner", "password123")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .admin_user_change_password(owner, "password123")
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
        .admin_user_change_password("owner", "password123")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![
    change_password,
    password_short,
    user_not_found,
    non_admin,
    no_token,
];
