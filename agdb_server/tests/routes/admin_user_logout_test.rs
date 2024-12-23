use crate::next_user_name;
use crate::TestServer;
use crate::ADMIN;

#[tokio::test]
async fn logout() -> anyhow::Result<()> {
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

#[tokio::test]
async fn unknown_user() -> anyhow::Result<()> {
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

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server.api.admin_user_logout(user).await.unwrap_err().status;
    assert_eq!(status, 401);

    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
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
