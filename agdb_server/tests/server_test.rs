mod framework;

use crate::framework::TestServer;
use assert_cmd::cargo::CommandCargoExt;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[tokio::test]
async fn db_reuse_and_error() -> anyhow::Result<()> {
    let mut server = TestServer::new()?;
    assert_eq!(server.get("/test_error").await?, 500);
    assert_eq!(server.get("/admin/shutdown").await?, 200);
    assert!(server.process.wait().unwrap().success());
    server.process = Command::cargo_bin("agdb_server")?
        .current_dir(&server.dir)
        .spawn()?;
    Ok(())
}

#[tokio::test]
async fn openapi() -> anyhow::Result<()> {
    let server = TestServer::new()?;
    assert_eq!(server.get("/openapi").await?, 200);
    Ok(())
}

#[tokio::test]
async fn create_user() -> anyhow::Result<()> {
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

#[tokio::test]
async fn add_database() -> anyhow::Result<()> {
    let server = TestServer::new()?;
    let mut user = HashMap::new();
    user.insert("name", "alice");
    user.insert("password", "mypassword123");
    assert_eq!(server.post("/user/create", &user).await?, 201); //created

    let mut db = HashMap::new();
    assert_eq!(server.post_auth("/db/add", "token", &db).await?, 401); //unauthorized

    let (status, token) = server.post_response("/user/login", &user).await?;
    assert_eq!(status, 200);

    db.insert("name", "mapped_db");
    db.insert("db_type", "memory");
    assert_eq!(server.post_auth("/db/add", &token, &db).await?, 201); //created
    db.insert("name", "file_db");
    db.insert("db_type", "file");
    assert_eq!(server.post_auth("/db/add", &token, &db).await?, 201); //created
    db.insert("name", "memory_db");
    db.insert("db_type", "mapped");
    assert_eq!(server.post_auth("/db/add", &token, &db).await?, 201); //created
    assert_eq!(server.post_auth("/db/add", &token, &db).await?, 403); //database exists
    db.insert("name", "");
    assert_eq!(server.post_auth("/db/add", &token, &db).await?, 461); //invalid db type

    Ok(())
}

#[tokio::test]
async fn list() -> anyhow::Result<()> {
    let server = TestServer::new()?;
    let mut user = HashMap::new();
    user.insert("name", "alice");
    user.insert("password", "mypassword123");
    assert_eq!(server.post("/user/create", &user).await?, 201); //created

    let (status, list) = server.get_auth_response("/db/list", "token").await?;
    assert_eq!(status, 401); //unauthorized
    assert!(list.is_empty());

    let (status, token) = server.post_response("/user/login", &user).await?;
    assert_eq!(status, 200);

    let (status, list) = server.get_auth_response("/db/list", &token).await?;
    assert_eq!(status, 200); //ok
    assert_eq!(list, "[]");

    let mut dbs = Vec::with_capacity(3);
    let mut db = HashMap::new();
    db.insert("name", "my_db");
    db.insert("db_type", "memory");
    dbs.push(db.clone());
    db.insert("name", "my_db2");
    db.insert("db_type", "file");
    dbs.push(db.clone());
    db.insert("name", "my_db3");
    db.insert("db_type", "mapped");
    dbs.push(db);

    assert_eq!(server.post_auth("/db/add", &token, &dbs[0]).await?, 201); //created
    assert_eq!(server.post_auth("/db/add", &token, &dbs[1]).await?, 201); //created
    assert_eq!(server.post_auth("/db/add", &token, &dbs[2]).await?, 201); //created

    let (status, list) = server.get_auth_response("/db/list", &token).await?;
    assert_eq!(status, 200); //ok

    let mut list: Vec<HashMap<&str, &str>> = serde_json::from_str(&list)?;
    list.sort_by(|left, right| left.get("name").unwrap().cmp(right.get("name").unwrap()));

    assert_eq!(dbs, list);

    assert!(!Path::new(&server.dir).join("my_db").exists());
    assert!(Path::new(&server.dir).join("my_db2").exists());
    assert!(Path::new(&server.dir).join("my_db3").exists());

    Ok(())
}

#[tokio::test]
async fn delete_db() -> anyhow::Result<()> {
    let server = TestServer::new()?;
    let mut user = HashMap::new();
    user.insert("name", "alice");
    user.insert("password", "mypassword123");
    assert_eq!(server.post("/user/create", &user).await?, 201); //created

    let mut delete_db = HashMap::new();
    delete_db.insert("name", "my_db");

    assert_eq!(
        server.post_auth("/db/delete", "token", &delete_db).await?,
        401
    ); //unauthorized

    let mut dbs = Vec::with_capacity(2);
    let mut db = HashMap::new();
    db.insert("name", "my_db");
    db.insert("db_type", "mapped");
    dbs.push(db.clone());
    db.insert("name", "my_db2");
    db.insert("db_type", "file");
    dbs.push(db);

    let (status, token) = server.post_response("/user/login", &user).await?;
    assert_eq!(status, 200); //ok

    assert_eq!(server.post_auth("/db/add", &token, &dbs[0]).await?, 201); //created
    assert_eq!(server.post_auth("/db/add", &token, &dbs[1]).await?, 201); //created

    assert!(Path::new(&server.dir).join("my_db").exists());
    assert!(Path::new(&server.dir).join("my_db2").exists());

    assert_eq!(
        server.post_auth("/db/delete", &token, &delete_db).await?,
        200
    );

    assert!(!Path::new(&server.dir).join("my_db").exists());
    assert!(Path::new(&server.dir).join("my_db2").exists());

    assert_eq!(
        server.post_auth("/db/delete", &token, &delete_db).await?,
        403
    );

    let (status, list) = server.get_auth_response("/db/list", &token).await?;
    assert_eq!(status, 200); //ok

    let list: Vec<HashMap<&str, &str>> = serde_json::from_str(&list)?;

    assert_eq!(dbs[1], list[0]);

    dbs[0].insert("db_type", "memory");

    assert_eq!(server.post_auth("/db/add", &token, &dbs[0]).await?, 201); //created

    assert!(!Path::new(&server.dir).join("my_db").exists());

    user.insert("name", "bob");
    assert_eq!(server.post("/user/create", &user).await?, 201); //created
    let (status, token2) = server.post_response("/user/login", &user).await?;
    assert_eq!(status, 200); //ok
    assert_eq!(
        server.post_auth("/db/delete", &token2, &delete_db).await?,
        403
    );
    assert_eq!(
        server.post_auth("/db/delete", &token, &delete_db).await?,
        200
    );

    Ok(())
}

#[tokio::test]
async fn remove_db() -> anyhow::Result<()> {
    let server = TestServer::new()?;
    let mut user = HashMap::new();
    user.insert("name", "alice");
    user.insert("password", "mypassword123");
    assert_eq!(server.post("/user/create", &user).await?, 201); //created

    let mut remove_db = HashMap::new();
    remove_db.insert("name", "my_db");

    assert_eq!(
        server.post_auth("/db/remove", "token", &remove_db).await?,
        401
    ); //unauthorized

    let mut dbs = Vec::with_capacity(2);
    let mut db = HashMap::new();
    db.insert("name", "my_db");
    db.insert("db_type", "mapped");
    dbs.push(db.clone());
    db.insert("name", "my_db2");
    db.insert("db_type", "file");
    dbs.push(db);

    let (status, token) = server.post_response("/user/login", &user).await?;
    assert_eq!(status, 200); //ok

    assert_eq!(server.post_auth("/db/add", &token, &dbs[0]).await?, 201); //created
    assert_eq!(server.post_auth("/db/add", &token, &dbs[1]).await?, 201); //created

    assert!(Path::new(&server.dir).join("my_db").exists());
    assert!(Path::new(&server.dir).join("my_db2").exists());

    assert_eq!(
        server.post_auth("/db/remove", &token, &remove_db).await?,
        200
    );

    assert!(Path::new(&server.dir).join("my_db").exists());
    assert!(Path::new(&server.dir).join("my_db2").exists());

    assert_eq!(
        server.post_auth("/db/remove", &token, &remove_db).await?,
        403
    );

    let (status, list) = server.get_auth_response("/db/list", &token).await?;
    assert_eq!(status, 200); //ok

    let list: Vec<HashMap<&str, &str>> = serde_json::from_str(&list)?;

    assert_eq!(dbs[1], list[0]);

    assert_eq!(server.post_auth("/db/add", &token, &dbs[0]).await?, 201); //created

    user.insert("name", "bob");
    assert_eq!(server.post("/user/create", &user).await?, 201); //created
    let (status, token2) = server.post_response("/user/login", &user).await?;
    assert_eq!(status, 200); //ok
    assert_eq!(
        server.post_auth("/db/remove", &token2, &remove_db).await?,
        403
    );
    assert_eq!(
        server.post_auth("/db/remove", &token, &remove_db).await?,
        200
    );

    assert!(Path::new(&server.dir).join("my_db").exists());

    Ok(())
}

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
