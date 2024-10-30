use crate::TestServer;
use crate::ADMIN;

#[tokio::test]
async fn user() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = &server.next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let user_status = server.api.user_status().await?.1;
    assert_eq!(user_status.name, *user);
    assert!(user_status.login);
    assert!(!user_status.admin);
    Ok(())
}

#[tokio::test]
async fn admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let user_status = server.api.user_status().await?.1;
    assert_eq!(user_status.name, ADMIN);
    assert!(user_status.login);
    assert!(user_status.admin);

    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server.api.user_status().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}
