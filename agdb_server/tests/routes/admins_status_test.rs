use crate::TestServer;
use crate::ADMIN;

#[tokio::test]
async fn status() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;

    let (status, admin_status) = server.api.admin_status().await?;
    assert_eq!(status, 200);
    assert_ne!(admin_status.uptime, 0);
    assert_eq!(admin_status.dbs, 0);
    assert_eq!(admin_status.users, 1);
    assert_eq!(admin_status.logged_in_users, 1);
    assert_ne!(admin_status.size, 0);

    Ok(())
}
