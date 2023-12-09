pub mod framework;

use crate::framework::TestServer;
use crate::framework::UserStatus;
use crate::framework::NO_TOKEN;
use crate::framework::USR_LIST_URI;

#[tokio::test]
async fn user_list() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.init_user("alice", "password123").await?;
    server.init_user("bob", "password456").await?;
    let admin = &server.admin_token;
    let (status, list) = server.get::<Vec<UserStatus>>(USR_LIST_URI, admin).await?;
    assert_eq!(status, 200);
    let mut list = list?;
    list.sort();
    let expected = vec![
        UserStatus {
            name: "admin".to_string(),
        },
        UserStatus {
            name: "alice".to_string(),
        },
        UserStatus {
            name: "bob".to_string(),
        },
    ];
    assert_eq!(list, expected);
    Ok(())
}

#[tokio::test]
async fn user_list_only_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let admin = server.init_admin().await?;
    let (status, list) = server.get::<Vec<UserStatus>>(USR_LIST_URI, &admin).await?;
    assert_eq!(status, 200);
    let expected = vec![UserStatus {
        name: "admin".to_string(),
    }];
    assert_eq!(list?, expected);
    Ok(())
}

#[tokio::test]
async fn user_list_no_admin_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (status, list) = server
        .get::<Vec<UserStatus>>(USR_LIST_URI, NO_TOKEN)
        .await?;
    assert_eq!(status, 401);
    assert!(list.is_err());
    Ok(())
}
