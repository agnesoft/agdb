pub mod framework;

use crate::framework::TestServer;
use crate::framework::User;
use crate::framework::ADMIN_USER_CREATE_URI;
use crate::framework::NO_TOKEN;

#[tokio::test]
async fn create_user() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = User {
        name: "alice",
        password: "password123",
    };
    let admin = server.init_admin().await?;
    assert_eq!(
        server.post(ADMIN_USER_CREATE_URI, &user, &admin).await?.0,
        201
    );
    Ok(())
}

#[tokio::test]
async fn name_too_short() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = User {
        name: "a",
        password: "password123",
    };
    let admin = server.init_admin().await?;
    assert_eq!(
        server.post(ADMIN_USER_CREATE_URI, &user, &admin).await?.0,
        462
    );
    Ok(())
}

#[tokio::test]
async fn password_too_short() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = User {
        name: "alice",
        password: "pswd",
    };
    let admin = server.init_admin().await?;
    assert_eq!(
        server.post(ADMIN_USER_CREATE_URI, &user, &admin).await?.0,
        461
    );
    Ok(())
}

#[tokio::test]
async fn user_already_exists() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = User {
        name: "alice",
        password: "password123",
    };
    server.init_user(user.name, user.password).await?;
    let admin = &server.admin_token;
    assert_eq!(
        server.post(ADMIN_USER_CREATE_URI, &user, admin).await?.0,
        463
    );
    Ok(())
}

#[tokio::test]
async fn no_admin_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = User {
        name: "alice",
        password: "password123",
    };
    assert_eq!(
        server.post(ADMIN_USER_CREATE_URI, &user, NO_TOKEN).await?.0,
        401
    );
    Ok(())
}
