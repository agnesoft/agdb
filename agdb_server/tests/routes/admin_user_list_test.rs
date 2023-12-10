use crate::framework::TestServer;
use crate::framework::UserStatus;
use crate::framework::ADMIN_USER_LIST_URI;
use crate::framework::NO_TOKEN;

#[tokio::test]
async fn user_list() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (status, list) = server
        .get::<Vec<UserStatus>>(ADMIN_USER_LIST_URI, &server.admin_token)
        .await?;
    assert_eq!(status, 200);
    assert!(list?.contains(&UserStatus {
        name: "admin".to_string()
    }));
    let (name1, _) = server.init_user().await?;
    let (name2, _) = server.init_user().await?;
    let (status, list) = server
        .get::<Vec<UserStatus>>(ADMIN_USER_LIST_URI, &server.admin_token)
        .await?;
    assert_eq!(status, 200);
    let list = list?;
    assert!(list.contains(&UserStatus {
        name: "admin".to_string(),
    }));
    assert!(list.contains(&UserStatus { name: name1 }));
    assert!(list.contains(&UserStatus { name: name2 }));
    Ok(())
}

#[tokio::test]
async fn no_admin_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (status, list) = server
        .get::<Vec<UserStatus>>(ADMIN_USER_LIST_URI, NO_TOKEN)
        .await?;
    assert_eq!(status, 401);
    assert!(list.is_err());
    Ok(())
}
