use crate::TestServer;
use crate::UserStatus;
use crate::ADMIN_USER_LIST_URI;
use crate::NO_TOKEN;

#[tokio::test]
async fn user_list() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user1 = server.init_user().await?;
    let user2 = server.init_user().await?;
    let (status, list) = server
        .get::<Vec<UserStatus>>(ADMIN_USER_LIST_URI, &server.admin_token)
        .await?;
    assert_eq!(status, 200);
    let list = list?;
    assert!(list.contains(&UserStatus {
        name: "admin".to_string()
    }));
    assert!(list.contains(&UserStatus { name: user1.name }));
    assert!(list.contains(&UserStatus { name: user2.name }));
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let (status, list) = server
        .get::<Vec<UserStatus>>(ADMIN_USER_LIST_URI, &user.token)
        .await?;
    assert_eq!(status, 401);
    assert!(list.is_err());
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (status, list) = server
        .get::<Vec<UserStatus>>(ADMIN_USER_LIST_URI, NO_TOKEN)
        .await?;
    assert_eq!(status, 401);
    assert!(list.is_err());
    Ok(())
}
