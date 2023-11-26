mod framework;

use crate::framework::TestServer;
use std::collections::HashMap;

#[tokio::test]
async fn config_port() -> anyhow::Result<()> {
    let server = TestServer::new()?;
    assert_eq!(server.get("/error").await?, 500);
    Ok(())
}

#[tokio::test]
async fn openapi() -> anyhow::Result<()> {
    let server = TestServer::new()?;
    assert_eq!(server.get("/openapi/").await?, 200);
    Ok(())
}

#[tokio::test]
async fn create_user() -> anyhow::Result<()> {
    let server = TestServer::new()?;
    let mut user = HashMap::new();
    user.insert("name", "a");
    user.insert("password", "");
    assert_eq!(server.post("/create_user", &user).await?, 461); //short name
    user.insert("name", "alice");
    assert_eq!(server.post("/create_user", &user).await?, 462); //short password
    user.insert("password", "mypassword123");
    assert_eq!(server.post("/create_user", &user).await?, 201); //created
    assert_eq!(server.post("/create_user", &user).await?, 463); //user exists
    Ok(())
}

#[tokio::test]
async fn login() -> anyhow::Result<()> {
    let server = TestServer::new()?;
    let mut user = HashMap::new();
    user.insert("name", "alice");
    user.insert("password", "mypassword123");
    assert_eq!(server.post("/login", &user).await?, 403); //unknown user
    assert_eq!(server.post("/create_user", &user).await?, 201); //created
    user.insert("password", "badpassword");
    assert_eq!(server.post("/login", &user).await?, 401); //bad password
    user.insert("password", "mypassword123");
    let (status, token) = server.post_response("/login", &user).await?;
    assert_eq!(status, 200); //login succeeded
    assert_eq!(token.len(), 38);
    Ok(())
}

#[tokio::test]
async fn create_database() -> anyhow::Result<()> {
    let server = TestServer::new()?;
    let mut user = HashMap::new();
    user.insert("name", "alice");
    user.insert("password", "mypassword123");
    let mut db = HashMap::new();
    db.insert("name", "my_db");
    db.insert("db_type", "mapped");
    assert_eq!(server.post_auth("/create_db", "", &db).await?, 401); //unauthorized
    assert_eq!(server.post("/create_user", &user).await?, 201); //created
    let (status, token) = server.post_response("/login", &user).await?;
    assert_eq!(status, 200);
    assert_eq!(server.post_auth("/create_db", &token, &db).await?, 201); //created
    assert_eq!(server.post_auth("/create_db", &token, &db).await?, 403); //database exists
    Ok(())
}
