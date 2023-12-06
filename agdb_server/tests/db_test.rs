pub mod framework;

use crate::framework::Db;
use crate::framework::TestServer;
use crate::framework::DB_ADD_URI;
use crate::framework::DB_DELETE_URI;
use crate::framework::DB_LIST_URI;
use crate::framework::DB_REMOVE_URI;
use crate::framework::DB_USER_ADD_URI;
use serde::Deserialize;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct DeleteDb<'a> {
    name: &'a str,
}

#[derive(Serialize, Deserialize)]
struct AddUser<'a> {
    user: &'a str,
    database: &'a str,
    role: &'a str,
}

#[tokio::test]
async fn add_database() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let token = server.init_user("alice", "mypassword123").await?;
    let bad_token = Some("bad".to_string());
    let db1 = Db {
        name: "db1".to_string(),
        db_type: "memory".to_string(),
    };
    let db2 = Db {
        name: "db2".to_string(),
        db_type: "file".to_string(),
    };
    let db3 = Db {
        name: "db3".to_string(),
        db_type: "mapped".to_string(),
    };
    let bad_name = Db {
        name: "".to_string(),
        db_type: "mapped".to_string(),
    };

    assert_eq!(server.post(DB_ADD_URI, &db1, &bad_token).await?.0, 401); //unauthorized
    assert_eq!(server.post(DB_ADD_URI, &db1, &token).await?.0, 201); //created
    assert_eq!(server.post(DB_ADD_URI, &db2, &token).await?.0, 201); //created
    assert_eq!(server.post(DB_ADD_URI, &db3, &token).await?.0, 201); //created
    assert_eq!(server.post(DB_ADD_URI, &db1, &token).await?.0, 403); //database exists
    assert_eq!(server.post(DB_ADD_URI, &bad_name, &token).await?.0, 461); //invalid db type

    Ok(())
}

#[tokio::test]
async fn add_user() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let token = server.init_user("alice", "mypassword123").await?;
    let token2: Option<String> = server.init_user("bob", "mypassword456").await?;
    let bad_token = Some("bad".to_string());
    let db = Db {
        name: "db1".to_string(),
        db_type: "mapped".to_string(),
    };
    let add_read = AddUser {
        database: "db1",
        user: "bob",
        role: "read",
    };
    let add_write = AddUser {
        database: "db1",
        user: "bob",
        role: "write",
    };
    let add_admin = AddUser {
        database: "db1",
        user: "bob",
        role: "admin",
    };
    let add_self = AddUser {
        database: "db1",
        user: "alice",
        role: "read",
    };
    let no_user = AddUser {
        database: "db1",
        user: "cat",
        role: "read",
    };
    let no_db = AddUser {
        database: "db_missing",
        user: "bob",
        role: "read",
    };
    assert_eq!(server.post(DB_ADD_URI, &db, &token).await?.0, 201); //created
    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token2).await?;
    assert_eq!(status, 200); //Ok
    let list = list?;
    assert_eq!(list, vec![]);
    assert_eq!(
        server.post(DB_USER_ADD_URI, &add_read, &bad_token).await?.0,
        401
    ); //forbidden
    assert_eq!(server.post(DB_USER_ADD_URI, &no_db, &token).await?.0, 461); //missing db
    assert_eq!(server.post(DB_USER_ADD_URI, &no_user, &token).await?.0, 462); //missing user
    assert_eq!(
        server.post(DB_USER_ADD_URI, &add_self, &token).await?.0,
        463
    ); //self
    assert_eq!(
        server.post(DB_USER_ADD_URI, &add_read, &token).await?.0,
        201
    ); //created
    assert_eq!(
        server.post(DB_USER_ADD_URI, &add_write, &token).await?.0,
        201
    ); //created
    assert_eq!(
        server.post(DB_USER_ADD_URI, &add_admin, &token).await?.0,
        201
    ); //created
    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token2).await?;
    let list = list?;
    assert_eq!(status, 200); //Ok
    assert_eq!(list, vec![db]);

    Ok(())
}

#[tokio::test]
async fn delete() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let token = server.init_user("alice", "mypassword123").await?;
    let bad_token = Some("bad".to_string());
    let db1 = Db {
        name: "db1".to_string(),
        db_type: "mapped".to_string(),
    };
    let db2 = Db {
        name: "db2".to_string(),
        db_type: "file".to_string(),
    };
    let db3 = Db {
        name: "db1".to_string(),
        db_type: "memory".to_string(),
    };
    let del1 = DeleteDb { name: "db1" };
    let del2 = DeleteDb { name: "db2" };

    assert_eq!(server.post(DB_DELETE_URI, &del1, &bad_token).await?.0, 401); //unauthorized
    assert_eq!(server.post(DB_ADD_URI, &db1, &token).await?.0, 201); //created
    assert_eq!(server.post(DB_ADD_URI, &db2, &token).await?.0, 201); //created

    assert!(Path::new(&server.dir).join(del1.name).exists());
    assert!(Path::new(&server.dir).join(del2.name).exists());

    assert_eq!(server.post(DB_DELETE_URI, &del1, &token).await?.0, 200);

    assert!(!Path::new(&server.dir).join(del1.name).exists());
    assert!(Path::new(&server.dir).join(del2.name).exists());

    assert_eq!(server.post(DB_DELETE_URI, &del1, &token).await?.0, 403); //cannot delete (already deleted)
    assert_eq!(server.post(DB_ADD_URI, &db3, &token).await?.0, 201); //created
    assert_eq!(server.post(DB_DELETE_URI, &del1, &token).await?.0, 200); //ok

    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    let mut list = list?;
    list.sort();
    assert_eq!(status, 200); //ok
    assert_eq!(list, vec![db2.clone()]);

    let token2 = server.init_user("bob", "mypassword456").await?;
    assert_eq!(server.post(DB_DELETE_URI, &db2, &token2).await?.0, 403); //forbidden

    Ok(())
}

#[tokio::test]
async fn list() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let token = server.init_user("alice", "mypassword123").await?;
    let bad = Some("bad".to_string());
    let db1 = Db {
        name: "db1".to_string(),
        db_type: "memory".to_string(),
    };
    let db2 = Db {
        name: "db2".to_string(),
        db_type: "memory".to_string(),
    };

    assert_eq!(server.get::<()>(DB_LIST_URI, &bad).await?.0, 401); //unauthorized
    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    assert_eq!(status, 200); //Ok
    assert_eq!(list?, vec![]);
    assert_eq!(server.post(DB_ADD_URI, &db1, &token).await?.0, 201); //created
    assert_eq!(server.post(DB_ADD_URI, &db2, &token).await?.0, 201); //created
    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    let mut list = list?;
    list.sort();
    assert_eq!(status, 200); //Ok
    assert_eq!(list, vec![db1, db2]);

    Ok(())
}

#[tokio::test]
async fn remove() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let token = server.init_user("alice", "mypassword123").await?;
    let bad_token = Some("bad".to_string());
    let db1 = Db {
        name: "db1".to_string(),
        db_type: "mapped".to_string(),
    };
    let db2 = Db {
        name: "db2".to_string(),
        db_type: "file".to_string(),
    };
    let del1 = DeleteDb { name: "db1" };
    let del2 = DeleteDb { name: "db2" };

    assert_eq!(server.post(DB_REMOVE_URI, &del1, &bad_token).await?.0, 401); //unauthorized
    assert_eq!(server.post(DB_ADD_URI, &db1, &token).await?.0, 201); //created
    assert_eq!(server.post(DB_ADD_URI, &db2, &token).await?.0, 201); //created

    assert!(Path::new(&server.dir).join(del1.name).exists());
    assert!(Path::new(&server.dir).join(del2.name).exists());

    assert_eq!(server.post(DB_REMOVE_URI, &del1, &token).await?.0, 200);

    assert!(Path::new(&server.dir).join(del1.name).exists());
    assert!(Path::new(&server.dir).join(del2.name).exists());

    assert_eq!(server.post(DB_REMOVE_URI, &del1, &token).await?.0, 403); //cannot delete (already deleted)

    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &token).await?;
    let mut list = list?;
    list.sort();
    assert_eq!(status, 200); //ok
    assert_eq!(list, vec![db2.clone()]);

    let token2 = server.init_user("bob", "mypassword456").await?;
    assert_eq!(server.post(DB_REMOVE_URI, &db2, &token2).await?.0, 403); //forbidden

    Ok(())
}
