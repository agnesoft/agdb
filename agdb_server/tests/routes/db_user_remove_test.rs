use crate::AddUser;
use crate::Db;
use crate::TestServer;
use crate::DB_LIST_URI;
use crate::DB_USER_ADD_URI;
use crate::DB_USER_REMOVE_URI;
use crate::NO_TOKEN;
use serde::Serialize;

#[derive(Serialize)]
struct RemoveUser<'a> {
    database: &'a str,
    user: &'a str,
}

#[tokio::test]
async fn remove() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let (reader_name, reader_token) = server.init_user().await?;
    let db = server.init_db("memory", &token).await?;
    let role = AddUser {
        database: &db,
        user: &reader_name,
        role: "read",
    };
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &reader_token).await?;
    assert_eq!(list?, vec![]);
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 201);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &reader_token).await?;
    let expected = vec![Db {
        name: db.clone(),
        db_type: "memory".to_string(),
    }];
    assert_eq!(list?, expected);
    let rem = RemoveUser {
        database: &db,
        user: &reader_name,
    };
    assert_eq!(server.post(DB_USER_REMOVE_URI, &rem, &token).await?.0, 204);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &reader_token).await?;
    assert_eq!(list?, vec![]);
    Ok(())
}

#[tokio::test]
async fn remove_user_non_admin() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (name, token) = server.init_user().await?;
    let (other_name, other) = server.init_user().await?;
    let db = server.init_db("memory", &token).await?;
    let role = AddUser {
        database: &db,
        user: &other_name,
        role: "read",
    };
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 201);
    let rem = RemoveUser {
        database: &db,
        user: &name,
    };
    assert_eq!(server.post(DB_USER_REMOVE_URI, &rem, &other).await?.0, 403);
    Ok(())
}

#[tokio::test]
async fn remove_self() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let (other_name, other) = server.init_user().await?;
    let db = server.init_db("memory", &token).await?;
    let role = AddUser {
        database: &db,
        user: &other_name,
        role: "read",
    };
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 201);
    let rem = RemoveUser {
        database: &db,
        user: &other_name,
    };
    assert_eq!(server.post(DB_USER_REMOVE_URI, &rem, &other).await?.0, 204);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &other).await?;
    assert_eq!(list?, vec![]);
    Ok(())
}

#[tokio::test]
async fn remove_self_admin() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let (other_name, other) = server.init_user().await?;
    let db = server.init_db("memory", &token).await?;
    let role = AddUser {
        database: &db,
        user: &other_name,
        role: "admin",
    };
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 201);
    let rem = RemoveUser {
        database: &db,
        user: &other_name,
    };
    assert_eq!(server.post(DB_USER_REMOVE_URI, &rem, &other).await?.0, 204);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &other).await?;
    assert_eq!(list?, vec![]);
    Ok(())
}

#[tokio::test]
async fn remove_self_admin_last() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (name, token) = server.init_user().await?;
    let db = server.init_db("memory", &token).await?;
    let rem = RemoveUser {
        database: &db,
        user: &name,
    };
    assert_eq!(server.post(DB_USER_REMOVE_URI, &rem, &token).await?.0, 403);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    let expected = vec![Db {
        name: db,
        db_type: "memory".to_string(),
    }];
    assert_eq!(list?, expected);
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let rem = RemoveUser {
        database: "db_not_found",
        user: "some_user",
    };
    assert_eq!(server.post(DB_USER_REMOVE_URI, &rem, &token).await?.0, 466);
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let db = server.init_db("memory", &token).await?;
    let rem = RemoveUser {
        database: &db,
        user: "user_not_found",
    };
    assert_eq!(server.post(DB_USER_REMOVE_URI, &rem, &token).await?.0, 464);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let rem = RemoveUser {
        database: "my_db",
        user: "bob",
    };
    assert_eq!(
        server.post(DB_USER_REMOVE_URI, &rem, NO_TOKEN).await?.0,
        401
    );
    Ok(())
}
