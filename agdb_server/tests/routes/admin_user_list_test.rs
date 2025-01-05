use crate::next_user_name;
use crate::TestServer;
use crate::ADMIN;
use agdb_api::UserStatus;

#[tokio::test]
async fn user_list() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user1 = &next_user_name();
    let user2 = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user1, user1).await?;
    server.api.admin_user_add(user2, user2).await?;
    let (status, list) = server.api.admin_user_list().await?;
    assert_eq!(status, 200);
    assert!(list.contains(&UserStatus {
        username: "admin".to_string(),
        login: true,
        admin: true,
    }));
    assert!(list.contains(&UserStatus {
        username: user1.to_string(),
        login: false,
        admin: false,
    }));
    assert!(list.contains(&UserStatus {
        username: user2.to_string(),
        login: false,
        admin: false,
    }));
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
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
