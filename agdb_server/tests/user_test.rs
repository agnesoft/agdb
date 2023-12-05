pub mod framework;

use crate::framework::ChangePassword;
use crate::framework::TestServer;
use crate::framework::User;
use crate::framework::CHANGE_PASSWORD_URI;
use crate::framework::CREATE_USER_URI;
use crate::framework::LOGIN_URI;
use crate::framework::NO_TOKEN;

#[tokio::test]
async fn chnage_password() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let admin = server.init_admin().await?;
    let bad = Some("bad".to_string());
    let user = User {
        name: "alice",
        password: "mypassword123",
    };
    let new_user = User {
        name: "alice",
        password: "mypassword456",
    };
    let change = ChangePassword {
        name: "alice",
        password: "mypassword123",
        new_password: "mypassword456",
    };
    let short = ChangePassword {
        name: "alice",
        password: "mypassword123",
        new_password: "pswd", //too short
    };

    assert_eq!(
        server.post(CHANGE_PASSWORD_URI, &short, NO_TOKEN).await?.0,
        462
    ); //password too short
    assert_eq!(
        server.post(CHANGE_PASSWORD_URI, &change, &bad).await?.0,
        403
    ); //fordbidden
    assert_eq!(server.post(CREATE_USER_URI, &user, &admin).await?.0, 201); //user created
    assert_eq!(
        server.post(CHANGE_PASSWORD_URI, &change, NO_TOKEN).await?.0,
        200
    ); //ok
    assert_eq!(
        server.post(CHANGE_PASSWORD_URI, &change, NO_TOKEN).await?.0,
        401
    ); //invalid password
    assert_eq!(server.post(LOGIN_URI, &user, NO_TOKEN).await?.0, 401); //invalid credentials
    assert_eq!(server.post(LOGIN_URI, &new_user, NO_TOKEN).await?.0, 200); //ok
    Ok(())
}

#[tokio::test]
async fn login() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let admin = server.init_admin().await?;
    let user = User {
        name: "alice",
        password: "mypassword123",
    };
    let bad_pswd = User {
        name: "alice",
        password: "mypassword456",
    };
    assert_eq!(server.post(LOGIN_URI, &user, NO_TOKEN).await?.0, 403); //unknown user
    assert_eq!(server.post(CREATE_USER_URI, &user, &admin).await?.0, 201); //created
    assert_eq!(server.post(LOGIN_URI, &bad_pswd, NO_TOKEN).await?.0, 401); //bad password
    let (status, token) = server.post("/user/login", &user, NO_TOKEN).await?;
    assert_eq!(status, 200); //user/login succeeded
    assert!(!token.is_empty());
    Ok(())
}
