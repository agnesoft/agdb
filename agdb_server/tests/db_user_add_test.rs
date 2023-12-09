pub mod framework;

use crate::framework::AddUser;
use crate::framework::Db;
use crate::framework::TestServer;
use crate::framework::DB_LIST_URI;
use crate::framework::DB_USER_ADD_URI;
use crate::framework::NO_TOKEN;

#[tokio::test]
async fn add_reader() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token = server.init_user("alice", "password123").await?;
    let reader = server.init_user("bob", "password465").await?;
    server.init_db("my_db", "memory", &token).await?;
    let role = AddUser {
        database: "my_db",
        user: "bob",
        role: "read",
    };
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &reader).await?;
    assert_eq!(list?, vec![]);
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 201);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &reader).await?;
    let expected = vec![Db {
        name: "my_db".to_string(),
        db_type: "memory".to_string(),
    }];
    assert_eq!(list?, expected);
    Ok(())
}

#[tokio::test]
async fn add_writer() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token = server.init_user("alice", "password123").await?;
    let reader = server.init_user("bob", "password465").await?;
    server.init_db("my_db", "memory", &token).await?;
    let role = AddUser {
        database: "my_db",
        user: "bob",
        role: "write",
    };
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &reader).await?;
    assert_eq!(list?, vec![]);
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 201);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &reader).await?;
    let expected = vec![Db {
        name: "my_db".to_string(),
        db_type: "memory".to_string(),
    }];
    assert_eq!(list?, expected);
    Ok(())
}

#[tokio::test]
async fn add_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token = server.init_user("alice", "password123").await?;
    let reader = server.init_user("bob", "password465").await?;
    server.init_db("my_db", "memory", &token).await?;
    let role = AddUser {
        database: "my_db",
        user: "bob",
        role: "admin",
    };
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &reader).await?;
    assert_eq!(list?, vec![]);
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 201);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &reader).await?;
    let expected = vec![Db {
        name: "my_db".to_string(),
        db_type: "memory".to_string(),
    }];
    assert_eq!(list?, expected);
    Ok(())
}

#[tokio::test]
async fn add_self() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token = server.init_user("alice", "password123").await?;
    server.init_db("my_db", "memory", &token).await?;
    let role = AddUser {
        database: "my_db",
        user: "alice",
        role: "read",
    };
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 403);
    Ok(())
}

#[tokio::test]
async fn add_admin_as_non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token = server.init_user("alice", "password123").await?;
    let writer = server.init_user("bob", "password465").await?;
    let other: Option<String> = server.init_user("chad", "password789").await?;
    server.init_db("my_db", "memory", &token).await?;
    let role = AddUser {
        database: "my_db",
        user: "chad",
        role: "write",
    };
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &other).await?;
    assert_eq!(list?, vec![]);
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &writer).await?.0, 403);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &other).await?;
    assert_eq!(list?, vec![]);
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token = server.init_user("alice", "password123").await?;
    let role = AddUser {
        database: "my_db",
        user: "bob",
        role: "read",
    };
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 466);
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token = server.init_user("alice", "password123").await?;
    server.init_db("my_db", "memory", &token).await?;
    let role = AddUser {
        database: "my_db",
        user: "bob",
        role: "read",
    };
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 464);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let role = AddUser {
        database: "my_db",
        user: "bob",
        role: "admin",
    };
    assert_eq!(server.post(DB_USER_ADD_URI, &role, NO_TOKEN).await?.0, 401);
    Ok(())
}
