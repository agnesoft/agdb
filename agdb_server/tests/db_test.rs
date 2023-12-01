pub mod framework;

use crate::framework::TestServer;
use std::collections::HashMap;
use std::panic::Location;
// use std::path::Path;

#[tokio::test]
async fn add_database() -> anyhow::Result<()> {
    let server = TestServer::new(100, Location::caller()).await?;
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

// #[tokio::test]
// async fn delete() -> anyhow::Result<()> {
//     let server = TestServer::new(100, Location::caller()).await?;
//     let mut user = HashMap::new();
//     user.insert("name", "alice");
//     user.insert("password", "mypassword123");
//     assert_eq!(server.post("/user/create", &user).await?, 201); //created

//     let mut delete_db = HashMap::new();
//     delete_db.insert("name", "my_db");

//     assert_eq!(
//         server.post_auth("/db/delete", "token", &delete_db).await?,
//         401
//     ); //unauthorized

//     let mut dbs = Vec::with_capacity(2);
//     let mut db = HashMap::new();
//     db.insert("name", "my_db");
//     db.insert("db_type", "mapped");
//     dbs.push(db.clone());
//     db.insert("name", "my_db2");
//     db.insert("db_type", "file");
//     dbs.push(db);

//     let (status, token) = server.post_response("/user/login", &user).await?;
//     assert_eq!(status, 200); //ok

//     assert_eq!(server.post_auth("/db/add", &token, &dbs[0]).await?, 201); //created
//     assert_eq!(server.post_auth("/db/add", &token, &dbs[1]).await?, 201); //created

//     assert!(Path::new(&server.dir).join("my_db").exists());
//     assert!(Path::new(&server.dir).join("my_db2").exists());

//     assert_eq!(
//         server.post_auth("/db/delete", &token, &delete_db).await?,
//         200
//     );

//     assert!(!Path::new(&server.dir).join("my_db").exists());
//     assert!(Path::new(&server.dir).join("my_db2").exists());

//     assert_eq!(
//         server.post_auth("/db/delete", &token, &delete_db).await?,
//         403
//     );

//     let (status, list) = server.get_auth_response("/db/list", &token).await?;
//     assert_eq!(status, 200); //ok

//     let list: Vec<HashMap<&str, &str>> = serde_json::from_str(&list)?;

//     assert_eq!(dbs[1], list[0]);

//     dbs[0].insert("db_type", "memory");

//     assert_eq!(server.post_auth("/db/add", &token, &dbs[0]).await?, 201); //created

//     assert!(!Path::new(&server.dir).join("my_db").exists());

//     user.insert("name", "bob");
//     assert_eq!(server.post("/user/create", &user).await?, 201); //created
//     let (status, token2) = server.post_response("/user/login", &user).await?;
//     assert_eq!(status, 200); //ok
//     assert_eq!(
//         server.post_auth("/db/delete", &token2, &delete_db).await?,
//         403
//     );
//     assert_eq!(
//         server.post_auth("/db/delete", &token, &delete_db).await?,
//         200
//     );

//     Ok(())
// }

// #[tokio::test]
// async fn list() -> anyhow::Result<()> {
//     let server = TestServer::new(100, Location::caller()).await?;
//     let mut user = HashMap::new();
//     user.insert("name", "alice");
//     user.insert("password", "mypassword123");
//     assert_eq!(server.post("/user/create", &user).await?, 201); //created

//     let (status, list) = server.get_auth_response("/db/list", "token").await?;
//     assert_eq!(status, 401); //unauthorized
//     assert!(list.is_empty());

//     let (status, token) = server.post_response("/user/login", &user).await?;
//     assert_eq!(status, 200);

//     let (status, list) = server.get_auth_response("/db/list", &token).await?;
//     assert_eq!(status, 200); //ok
//     assert_eq!(list, "[]");

//     let mut dbs = Vec::with_capacity(3);
//     let mut db = HashMap::new();
//     db.insert("name", "my_db");
//     db.insert("db_type", "memory");
//     dbs.push(db.clone());
//     db.insert("name", "my_db2");
//     db.insert("db_type", "file");
//     dbs.push(db.clone());
//     db.insert("name", "my_db3");
//     db.insert("db_type", "mapped");
//     dbs.push(db);

//     assert_eq!(server.post_auth("/db/add", &token, &dbs[0]).await?, 201); //created
//     assert_eq!(server.post_auth("/db/add", &token, &dbs[1]).await?, 201); //created
//     assert_eq!(server.post_auth("/db/add", &token, &dbs[2]).await?, 201); //created

//     let (status, list) = server.get_auth_response("/db/list", &token).await?;
//     assert_eq!(status, 200); //ok

//     let mut list: Vec<HashMap<&str, &str>> = serde_json::from_str(&list)?;
//     list.sort_by(|left, right| left.get("name").unwrap().cmp(right.get("name").unwrap()));

//     assert_eq!(dbs, list);

//     assert!(!Path::new(&server.dir).join("my_db").exists());
//     assert!(Path::new(&server.dir).join("my_db2").exists());
//     assert!(Path::new(&server.dir).join("my_db3").exists());

//     Ok(())
// }

// #[tokio::test]
// async fn remove() -> anyhow::Result<()> {
//     let server = TestServer::new(100, Location::caller()).await?;
//     let mut user = HashMap::new();
//     user.insert("name", "alice");
//     user.insert("password", "mypassword123");
//     assert_eq!(server.post("/user/create", &user).await?, 201); //created

//     let mut remove_db = HashMap::new();
//     remove_db.insert("name", "my_db");

//     assert_eq!(
//         server.post_auth("/db/remove", "token", &remove_db).await?,
//         401
//     ); //unauthorized

//     let mut dbs = Vec::with_capacity(2);
//     let mut db = HashMap::new();
//     db.insert("name", "my_db");
//     db.insert("db_type", "mapped");
//     dbs.push(db.clone());
//     db.insert("name", "my_db2");
//     db.insert("db_type", "file");
//     dbs.push(db);

//     let (status, token) = server.post_response("/user/login", &user).await?;
//     assert_eq!(status, 200); //ok

//     assert_eq!(server.post_auth("/db/add", &token, &dbs[0]).await?, 201); //created
//     assert_eq!(server.post_auth("/db/add", &token, &dbs[1]).await?, 201); //created

//     assert!(Path::new(&server.dir).join("my_db").exists());
//     assert!(Path::new(&server.dir).join("my_db2").exists());

//     assert_eq!(
//         server.post_auth("/db/remove", &token, &remove_db).await?,
//         200
//     );

//     assert!(Path::new(&server.dir).join("my_db").exists());
//     assert!(Path::new(&server.dir).join("my_db2").exists());

//     assert_eq!(
//         server.post_auth("/db/remove", &token, &remove_db).await?,
//         403
//     );

//     let (status, list) = server.get_auth_response("/db/list", &token).await?;
//     assert_eq!(status, 200); //ok

//     let list: Vec<HashMap<&str, &str>> = serde_json::from_str(&list)?;

//     assert_eq!(dbs[1], list[0]);

//     assert_eq!(server.post_auth("/db/add", &token, &dbs[0]).await?, 201); //created

//     user.insert("name", "bob");
//     assert_eq!(server.post("/user/create", &user).await?, 201); //created
//     let (status, token2) = server.post_response("/user/login", &user).await?;
//     assert_eq!(status, 200); //ok
//     assert_eq!(
//         server.post_auth("/db/remove", &token2, &remove_db).await?,
//         403
//     );
//     assert_eq!(
//         server.post_auth("/db/remove", &token, &remove_db).await?,
//         200
//     );

//     assert!(Path::new(&server.dir).join("my_db").exists());

//     Ok(())
// }
