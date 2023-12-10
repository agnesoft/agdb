use crate::framework::AddUser;
use crate::framework::Db;
use crate::framework::TestServer;
use crate::framework::DB_DELETE_URI;
use crate::framework::DB_LIST_URI;
use crate::framework::DB_USER_ADD_URI;
use crate::framework::NO_TOKEN;
use serde::Deserialize;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct DeleteDb<'a> {
    name: &'a str,
}

#[tokio::test]
async fn delete() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let db = server.init_db("mapped", &token).await?;
    assert!(Path::new(&server.dir).join(&db).exists());
    let del = DeleteDb { name: &db };
    assert_eq!(server.post(DB_DELETE_URI, &del, &token).await?.0, 204);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    assert_eq!(list?, vec![]);
    assert!(!Path::new(&server.dir).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let del = DeleteDb {
        name: "delete_db_not_found",
    };
    assert_eq!(server.post(DB_DELETE_URI, &del, &token).await?.0, 466);
    Ok(())
}

#[tokio::test]
async fn other_user() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let db = server.init_db("mapped", &token).await?;
    let (_, other) = server.init_user().await?;
    let del = DeleteDb { name: &db };
    assert_eq!(server.post(DB_DELETE_URI, &del, &other).await?.0, 466);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    let expected = vec![Db {
        name: db.clone(),
        db_type: "mapped".to_string(),
    }];
    assert_eq!(list?, expected);
    assert!(Path::new(&server.dir).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn with_read_role() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let db = server.init_db("mapped", &token).await?;
    let (reader_name, reader_token) = server.init_user().await?;
    let role = AddUser {
        database: &db,
        user: &reader_name,
        role: "read",
    };
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 201);
    let del = DeleteDb { name: &db };
    assert_eq!(
        server.post(DB_DELETE_URI, &del, &reader_token).await?.0,
        403
    );
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    let expected = vec![Db {
        name: db.clone(),
        db_type: "mapped".to_string(),
    }];
    assert_eq!(list?, expected);
    assert!(Path::new(&server.dir).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn with_write_role() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let db = server.init_db("mapped", &token).await?;
    let (writer_name, writer_token) = server.init_user().await?;
    let role = AddUser {
        database: &db,
        user: &writer_name,
        role: "write",
    };
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 201);
    let del = DeleteDb { name: &db };
    assert_eq!(
        server.post(DB_DELETE_URI, &del, &writer_token).await?.0,
        403
    );
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    let expected = vec![Db {
        name: db.clone(),
        db_type: "mapped".to_string(),
    }];
    assert_eq!(list?, expected);
    assert!(Path::new(&server.dir).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn with_admin_role() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let db = server.init_db("memory", &token).await?;
    let (admin_name, admin_token) = server.init_user().await?;
    let role = AddUser {
        database: &db,
        user: &admin_name,
        role: "admin",
    };
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 201);
    let del = DeleteDb { name: &db };
    assert_eq!(server.post(DB_DELETE_URI, &del, &admin_token).await?.0, 204);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    assert_eq!(list?, vec![]);
    assert!(!Path::new(&server.dir).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let del = DeleteDb {
        name: "no_token_db",
    };
    assert_eq!(server.post(DB_DELETE_URI, &del, NO_TOKEN).await?.0, 401);
    Ok(())
}
