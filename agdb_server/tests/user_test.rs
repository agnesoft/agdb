pub mod framework;

use crate::framework::TestServer;
use crate::framework::NO_TOKEN;
use std::collections::HashMap;

#[tokio::test]
async fn chnage_password() -> anyhow::Result<()> {
    let server = TestServer::new().await?;

    let mut user = HashMap::<&str, &str>::new();
    user.insert("name", "alice");
    user.insert("password", "mypassword123");

    let mut change = HashMap::new();
    change.insert("name", "alice");
    change.insert("password", "mypassword123");
    change.insert("new_password", "pswd");

    assert_eq!(
        server
            .post("/user/change_password", &change, NO_TOKEN)
            .await?
            .0,
        462
    ); //password too short

    change.insert("new_password", "mypassword456");
    let bad_token = Some("bad".to_string());
    assert_eq!(
        server
            .post("/user/change_password", &change, &bad_token)
            .await?
            .0,
        403
    ); //fordbidden

    let admin_token = server.init_admin().await?;
    assert_eq!(
        server
            .post("/admin/create_user", &user, &admin_token)
            .await?
            .0,
        201
    ); //user created
    assert_eq!(
        server
            .post("/user/change_password", &change, NO_TOKEN)
            .await?
            .0,
        200
    ); //ok
    assert_eq!(
        server
            .post("/user/change_password", &change, NO_TOKEN)
            .await?
            .0,
        401
    ); //invalid password

    assert_eq!(server.post("/user/login", &user, NO_TOKEN).await?.0, 401); //invalid credentials
    user.insert("password", "mypassword456");
    assert_eq!(server.post("/user/login", &user, NO_TOKEN).await?.0, 200); //ok
    Ok(())
}

#[tokio::test]
async fn login() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let mut user = HashMap::new();
    user.insert("name", "alice");
    user.insert("password", "mypassword123");
    assert_eq!(server.post("/user/login", &user, NO_TOKEN).await?.0, 403); //unknown user
    let admin_token = server.init_admin().await?;
    assert_eq!(
        server
            .post("/admin/create_user", &user, &admin_token)
            .await?
            .0,
        201
    ); //created
    user.insert("password", "badpassword");
    assert_eq!(server.post("/user/login", &user, NO_TOKEN).await?.0, 401); //bad password
    user.insert("password", "mypassword123");
    let (status, token) = server.post("/user/login", &user, NO_TOKEN).await?;
    assert_eq!(status, 200); //user/login succeeded
    assert!(!token.is_empty());
    Ok(())
}
