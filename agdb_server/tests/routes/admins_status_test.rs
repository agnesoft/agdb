use crate::TestServer;
use crate::ADMIN;

#[tokio::test]
async fn status() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;

    let (status, admin_status) = server.api.admin_status().await?;
    assert_eq!(status, 200);
    assert_eq!(admin_status.dbs, 0);
    assert_eq!(admin_status.users, 1);
    assert_eq!(admin_status.logged_in_users, 1);
    assert_ne!(admin_status.size, 0);

    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = &server.next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server.api.admin_status().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server.api.admin_status().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}
