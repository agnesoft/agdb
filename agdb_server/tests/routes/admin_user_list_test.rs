use crate::TestServer;
use crate::ADMIN;
use agdb_api::UserStatus;

#[tokio::test]
async fn user_list() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user1 = &server.next_user_name();
    let user2 = &server.next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user1, user1).await?;
    server.api.admin_user_add(user2, user2).await?;
    let (status, list) = server.api.admin_user_list().await?;
    assert_eq!(status, 200);
    assert!(list.contains(&UserStatus {
        name: "admin".to_string(),
        login: true
    }));
    assert!(list.contains(&UserStatus {
        name: user1.to_string(),
        login: false,
    }));
    assert!(list.contains(&UserStatus {
        name: user2.to_string(),
        login: false,
    }));
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = &server.next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server.api.admin_user_list().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server.api.admin_user_list().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}
