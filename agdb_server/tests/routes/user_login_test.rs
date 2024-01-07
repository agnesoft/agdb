use crate::TestServer;
use crate::ADMIN;

#[tokio::test]
async fn login() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let status = server.api.user_login(ADMIN, ADMIN).await?;
    assert_eq!(status, 200);
    server.api.admin_user_add(owner, owner).await?;
    let status = server.api.user_login(owner, owner).await?;
    assert_eq!(status, 200);
    assert!(!server.api.token.clone().unwrap().is_empty());
    Ok(())
}

#[tokio::test]
async fn repeated_login() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let status = server.api.user_login(ADMIN, ADMIN).await?;
    assert_eq!(status, 200);
    server.api.admin_user_add(owner, owner).await?;
    let status = server.api.user_login(owner, owner).await?;
    assert_eq!(status, 200);
    let token = server.api.token.clone().unwrap();
    let status = server.api.user_login(owner, owner).await?;
    assert_eq!(status, 200);
    assert_eq!(server.api.token.clone().unwrap(), token);
    Ok(())
}

#[tokio::test]
async fn invalid_credentials() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let status = server.api.user_login(ADMIN, ADMIN).await?;
    assert_eq!(status, 200);
    server.api.admin_user_add(owner, owner).await?;
    let status = server
        .api
        .user_login(owner, "bad_password")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let status = server
        .api
        .user_login("owner", "bad_password")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}
