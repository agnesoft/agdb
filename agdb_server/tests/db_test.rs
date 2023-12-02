pub mod framework;

use crate::framework::TestServer;
use std::collections::HashMap;
use std::path::Path;

#[tokio::test]
async fn add_database() -> anyhow::Result<()> {
    let server = TestServer::new().await?;

    let mut db = HashMap::new();
    let bad_token = Some("bad".to_string());
    assert_eq!(server.post("/db/add", &db, &bad_token).await?.0, 401); //unauthorized

    let token = server.init_user("alice", "mypassword123").await?;
    db.insert("name", "mapped_db");
    db.insert("db_type", "memory");
    assert_eq!(server.post("/db/add", &db, &token).await?.0, 201); //created
    db.insert("name", "file_db");
    db.insert("db_type", "file");
    assert_eq!(server.post("/db/add", &db, &token).await?.0, 201); //created
    db.insert("name", "memory_db");
    db.insert("db_type", "mapped");
    assert_eq!(server.post("/db/add", &db, &token).await?.0, 201); //created
    assert_eq!(server.post("/db/add", &db, &token).await?.0, 403); //database exists
    db.insert("name", "");
    assert_eq!(server.post("/db/add", &db, &token).await?.0, 461); //invalid db type

    Ok(())
}

#[tokio::test]
async fn delete() -> anyhow::Result<()> {
    let server = TestServer::new().await?;

    let mut delete_db = HashMap::new();
    delete_db.insert("name", "my_db");

    let bad_token = Some("bad".to_string());
    assert_eq!(
        server.post("/db/delete", &delete_db, &bad_token).await?.0,
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

    let token = server.init_user("alice", "mypassword123").await?;
    assert_eq!(server.post("/db/add", &dbs[0], &token).await?.0, 201); //created
    assert_eq!(server.post("/db/add", &dbs[1], &token).await?.0, 201); //created

    assert!(Path::new(&server.dir).join("my_db").exists());
    assert!(Path::new(&server.dir).join("my_db2").exists());

    assert_eq!(server.post("/db/delete", &delete_db, &token).await?.0, 200);

    assert!(!Path::new(&server.dir).join("my_db").exists());
    assert!(Path::new(&server.dir).join("my_db2").exists());

    assert_eq!(server.post("/db/delete", &delete_db, &token).await?.0, 403);

    let (status, list) = server.get("/db/list", &token).await?;
    assert_eq!(status, 200); //ok
    let list: Vec<HashMap<&str, &str>> = serde_json::from_str(&list)?;
    assert_eq!(dbs[1], list[0]);

    dbs[0].insert("db_type", "memory");
    assert_eq!(server.post("/db/add", &dbs[0], &token).await?.0, 201); //created

    assert!(!Path::new(&server.dir).join("my_db").exists());

    let token2 = server.init_user("bob", "mypassword456").await?;
    assert_eq!(server.post("/db/delete", &delete_db, &token2).await?.0, 403); //forbidden
    assert_eq!(server.post("/db/delete", &delete_db, &token).await?.0, 200); // deleted

    Ok(())
}

#[tokio::test]
async fn list() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let token = server.init_user("alice", "mypassword123").await?;

    let bad_token = Some("bad".to_string());
    assert_eq!(server.get("/db/list", &bad_token).await?.0, 401); //unauthorized

    let (status, list) = server.get("/db/list", &token).await?;
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

    assert_eq!(server.post("/db/add", &dbs[0], &token).await?.0, 201); //created
    assert_eq!(server.post("/db/add", &dbs[1], &token).await?.0, 201); //created
    assert_eq!(server.post("/db/add", &dbs[2], &token).await?.0, 201); //created

    let (status, list) = server.get("/db/list", &token).await?;
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
async fn remove() -> anyhow::Result<()> {
    let server = TestServer::new().await?;

    let mut remove_db = HashMap::new();
    remove_db.insert("name", "my_db");

    let bad_token = Some("bad".to_string());
    assert_eq!(
        server.post("/db/remove", &remove_db, &bad_token).await?.0,
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

    let token = server.init_user("alice", "mypassword123").await?;
    assert_eq!(server.post("/db/add", &dbs[0], &token).await?.0, 201); //created
    assert_eq!(server.post("/db/add", &dbs[1], &token).await?.0, 201); //created

    assert!(Path::new(&server.dir).join("my_db").exists());
    assert!(Path::new(&server.dir).join("my_db2").exists());

    assert_eq!(server.post("/db/remove", &remove_db, &token).await?.0, 200); //removed

    assert!(Path::new(&server.dir).join("my_db").exists());
    assert!(Path::new(&server.dir).join("my_db2").exists());

    assert_eq!(server.post("/db/remove", &remove_db, &token).await?.0, 403); //forbidden

    let (status, list) = server.get("/db/list", &token).await?;
    assert_eq!(status, 200); //ok
    let list: Vec<HashMap<&str, &str>> = serde_json::from_str(&list)?;
    assert_eq!(dbs[1], list[0]);

    dbs[0].insert("db_type", "memory");
    assert_eq!(server.post("/db/add", &dbs[0], &token).await?.0, 201); //created

    assert!(Path::new(&server.dir).join("my_db").exists());

    let token2 = server.init_user("bob", "mypassword456").await?;
    assert_eq!(server.post("/db/remove", &remove_db, &token2).await?.0, 403); //forbidden
    assert_eq!(server.post("/db/remove", &remove_db, &token).await?.0, 200); // removed

    Ok(())
}
