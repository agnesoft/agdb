pub mod framework;

use crate::framework::TestServer;
use crate::framework::User;
use crate::framework::CREATE_USER_URI;
use crate::framework::NO_TOKEN;

#[tokio::test]
async fn create_user() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = User {
        name: "alice",
        password: "password123",
    };
    let admin = server.init_admin().await?;
    assert_eq!(server.post(CREATE_USER_URI, &user, &admin).await?.0, 201);
    Ok(())
}

#[tokio::test]
async fn create_user_short_name() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = User {
        name: "a",
        password: "password123",
    };
    let admin = server.init_admin().await?;
    assert_eq!(server.post(CREATE_USER_URI, &user, &admin).await?.0, 462);
    Ok(())
}

#[tokio::test]
async fn create_user_short_password() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = User {
        name: "alice",
        password: "pswd",
    };
    let admin = server.init_admin().await?;
    assert_eq!(server.post(CREATE_USER_URI, &user, &admin).await?.0, 461);
    Ok(())
}

#[tokio::test]
async fn create_user_user_exists() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = User {
        name: "alice",
        password: "password123",
    };
    server.init_user(user.name, user.password).await?;
    let admin = &server.admin_token;
    assert_eq!(server.post(CREATE_USER_URI, &user, admin).await?.0, 463);
    Ok(())
}

#[tokio::test]
async fn create_user_no_admin_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = User {
        name: "alice",
        password: "password123",
    };
    assert_eq!(server.post(CREATE_USER_URI, &user, NO_TOKEN).await?.0, 401);
    Ok(())
}

#[tokio::test]
async fn create_user_bad_admin_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = User {
        name: "alice",
        password: "password123",
    };
    let admin = Some("bad".to_string());
    assert_eq!(server.post(CREATE_USER_URI, &user, &admin).await?.0, 401);
    Ok(())
}
