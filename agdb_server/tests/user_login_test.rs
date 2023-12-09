pub mod framework;

use crate::framework::TestServer;
use crate::framework::User;
use crate::framework::NO_TOKEN;
use crate::framework::USER_LOGIN_URI;

#[tokio::test]
async fn login() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.init_user("alice", "password123").await?;
    let user = User {
        name: "alice",
        password: "password123",
    };
    let (status, token) = server.post(USER_LOGIN_URI, &user, NO_TOKEN).await?;
    assert_eq!(status, 200);
    assert!(!token.is_empty());

    Ok(())
}

#[tokio::test]
async fn invalid_credentials() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.init_user("alice", "password123").await?;
    let user = User {
        name: "alice",
        password: "password456",
    };
    let (status, token) = server.post(USER_LOGIN_URI, &user, NO_TOKEN).await?;
    assert_eq!(status, 401);
    assert!(token.is_empty());

    Ok(())
}
