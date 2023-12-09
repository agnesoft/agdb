pub mod framework;

use crate::framework::AddUser;
use crate::framework::Db;
use crate::framework::TestServer;
use crate::framework::DB_LIST_URI;
use crate::framework::DB_REMOVE_URI;
use crate::framework::DB_USER_ADD_URI;
use crate::framework::NO_TOKEN;
use serde::Deserialize;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct RemoveDb<'a> {
    name: &'a str,
}

#[tokio::test]
async fn delete() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token = server.init_user("alice", "password123").await?;
    server.init_db("my_db", "mapped", &token).await?;
    assert!(Path::new(&server.dir).join("my_db").exists());
    let del = RemoveDb { name: "my_db" };
    assert_eq!(server.post(DB_REMOVE_URI, &del, &token).await?.0, 204);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    assert_eq!(list?, vec![]);
    assert!(Path::new(&server.dir).join("my_db").exists());
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token = server.init_user("alice", "password123").await?;
    let del = RemoveDb { name: "my_db" };
    assert_eq!(server.post(DB_REMOVE_URI, &del, &token).await?.0, 466);
    Ok(())
}

#[tokio::test]
async fn other_user() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token = server.init_user("alice", "password123").await?;
    server.init_db("my_db", "mapped", &token).await?;
    let other = server.init_user("bob", "password456").await?;
    let del = RemoveDb { name: "my_db" };
    assert_eq!(server.post(DB_REMOVE_URI, &del, &other).await?.0, 466);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    let expected = vec![Db {
        name: "my_db".to_string(),
        db_type: "mapped".to_string(),
    }];
    assert_eq!(list?, expected);
    Ok(())
}

#[tokio::test]
async fn with_read_role() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token = server.init_user("alice", "password123").await?;
    server.init_db("my_db", "mapped", &token).await?;
    let reader = server.init_user("bob", "password456").await?;
    let role = AddUser {
        database: "my_db",
        user: "bob",
        role: "read",
    };
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 201);
    let del = RemoveDb { name: "my_db" };
    assert_eq!(server.post(DB_REMOVE_URI, &del, &reader).await?.0, 403);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    let expected = vec![Db {
        name: "my_db".to_string(),
        db_type: "mapped".to_string(),
    }];
    assert_eq!(list?, expected);
    Ok(())
}

#[tokio::test]
async fn with_write_role() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token = server.init_user("alice", "password123").await?;
    server.init_db("my_db", "mapped", &token).await?;
    let reader = server.init_user("bob", "password456").await?;
    let role = AddUser {
        database: "my_db",
        user: "bob",
        role: "write",
    };
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 201);
    let del = RemoveDb { name: "my_db" };
    assert_eq!(server.post(DB_REMOVE_URI, &del, &reader).await?.0, 403);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    let expected = vec![Db {
        name: "my_db".to_string(),
        db_type: "mapped".to_string(),
    }];
    assert_eq!(list?, expected);
    Ok(())
}

#[tokio::test]
async fn with_admin_role() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token = server.init_user("alice", "password123").await?;
    server.init_db("my_db", "mapped", &token).await?;
    let reader = server.init_user("bob", "password456").await?;
    let role = AddUser {
        database: "my_db",
        user: "bob",
        role: "admin",
    };
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 201);
    let del = RemoveDb { name: "my_db" };
    assert_eq!(server.post(DB_REMOVE_URI, &del, &reader).await?.0, 204);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    assert_eq!(list?, vec![]);
    assert!(Path::new(&server.dir).join("my_db").exists());
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let del = RemoveDb { name: "my_db" };
    assert_eq!(server.post(DB_REMOVE_URI, &del, NO_TOKEN).await?.0, 401);
    Ok(())
}
