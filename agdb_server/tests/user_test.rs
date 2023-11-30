pub mod framework;

use crate::framework::TestServer;
use std::collections::HashMap;

#[tokio::test]
async fn chnage_password() -> anyhow::Result<()> {
    let server = TestServer::new()?;
    let mut user = HashMap::new();
    user.insert("name", "alice");
    user.insert("password", "mypassword123");
    let mut change = HashMap::new();
    change.insert("name", "alice");
    change.insert("password", "mypassword123");
    change.insert("new_password", "pswd");
    assert_eq!(server.post("/user/change_password", &change).await?, 462);
    change.insert("new_password", "mypassword456");
    assert_eq!(server.post("/user/change_password", &change).await?, 403);
    assert_eq!(server.post("/user/create", &user).await?, 201); //created
    let (status, token) = server.post_response("/user/login", &user).await?;
    assert_eq!(status, 200); //user/login succeeded
    assert_eq!(token.len(), 38);
    assert_eq!(server.post("/user/change_password", &change).await?, 200); //ok
    assert_eq!(server.post("/user/change_password", &change).await?, 401); //invalid password
    assert_eq!(server.post("/user/login", &user).await?, 401); //invalid credentials
    user.insert("password", "mypassword456");
    assert_eq!(server.post("/user/login", &user).await?, 200); //ok
    Ok(())
}

#[tokio::test]
async fn create() -> anyhow::Result<()> {
    let server = TestServer::new()?;
    let mut user = HashMap::new();
    user.insert("name", "a");
    user.insert("password", "");
    assert_eq!(server.post("/user/create", &user).await?, 461); //short name
    user.insert("name", "alice");
    assert_eq!(server.post("/user/create", &user).await?, 462); //short password
    user.insert("password", "mypassword123");
    assert_eq!(server.post("/user/create", &user).await?, 201); //created
    assert_eq!(server.post("/user/create", &user).await?, 463); //user exists
    Ok(())
}

#[tokio::test]
async fn login() -> anyhow::Result<()> {
    let server = TestServer::new()?;
    let mut user = HashMap::new();
    user.insert("name", "alice");
    user.insert("password", "mypassword123");
    assert_eq!(server.post("/user/login", &user).await?, 403); //unknown user
    assert_eq!(server.post("/user/create", &user).await?, 201); //created
    user.insert("password", "badpassword");
    assert_eq!(server.post("/user/login", &user).await?, 401); //bad password
    user.insert("password", "mypassword123");
    let (status, token) = server.post_response("/user/login", &user).await?;
    assert_eq!(status, 200); //user/login succeeded
    assert_eq!(token.len(), 38);
    Ok(())
}
