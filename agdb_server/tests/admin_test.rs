pub mod framework;

use crate::framework::TestServer;
use crate::framework::NO_TOKEN;
use std::collections::HashMap;

#[tokio::test]
async fn create_user() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let mut user = HashMap::new();
    user.insert("name", "a");
    user.insert("password", "");
    assert_eq!(
        server.post("/admin/user/create", &user, NO_TOKEN).await?.0,
        401
    ); //permission denied
    let bad_token = Some("bad".to_string());
    assert_eq!(
        server
            .post("/admin/user/create", &user, &bad_token)
            .await?
            .0,
        403
    ); //forbidden
    let admin_token = server.init_admin().await?;
    assert_eq!(
        server
            .post("/admin/user/create", &user, &admin_token)
            .await?
            .0,
        461
    ); //short name
    user.insert("name", "alice");
    assert_eq!(
        server
            .post("/admin/user/create", &user, &admin_token)
            .await?
            .0,
        462
    ); //short password
    user.insert("password", "mypassword123");
    assert_eq!(
        server
            .post("/admin/user/create", &user, &admin_token)
            .await?
            .0,
        201
    ); //created
    assert_eq!(
        server
            .post("/admin/user/create", &user, &admin_token)
            .await?
            .0,
        463
    ); //user exists
    Ok(())
}

#[tokio::test]
async fn change_password() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let admin_token = server.init_admin().await?;

    let mut user = HashMap::new();
    user.insert("name", "alice");
    user.insert("password", "password123");

    assert_eq!(
        server
            .post("/admin/user/create", &user, &admin_token)
            .await?
            .0,
        201
    );

    user.insert("password", "password456");

    let bad_token = Some("bad".to_string());
    assert_eq!(
        server
            .post("/admin/user/change_password", &user, &bad_token)
            .await?
            .0,
        403
    );

    assert_eq!(
        server
            .post("/admin/user/change_password", &user, &admin_token)
            .await?
            .0,
        200
    );

    assert_eq!(server.post("/user/login", &user, NO_TOKEN).await?.0, 200);

    user.insert("name", "bob");

    assert_eq!(
        server
            .post("/admin/user/change_password", &user, &admin_token)
            .await?
            .0,
        403
    );

    user.insert("password", "pass");

    assert_eq!(
        server
            .post("/admin/user/change_password", &user, &admin_token)
            .await?
            .0,
        462
    );

    Ok(())
}

#[tokio::test]
async fn db_list() -> anyhow::Result<()> {
    let server = TestServer::new().await?;

    let bad_token = Some("bad".to_string());
    assert_eq!(server.get("/admin/db/list", &bad_token).await?.0, 403);

    let admin_token = server.init_admin().await?;
    let (status, list) = server.get("/admin/db/list", &admin_token).await?;
    let list: Vec<HashMap<&str, &str>> = serde_json::from_str(&list)?;
    assert_eq!(status, 200);
    assert!(list.is_empty());

    let mut token = server.init_user("alice", "password123").await?;
    let mut db = HashMap::<&str, &str>::new();
    db.insert("name", "db1");
    db.insert("db_type", "memory");
    assert_eq!(server.post("/db/add", &db, &token).await?.0, 201);

    token = server.init_user("bob", "password456").await?;
    db.insert("name", "db2");
    assert_eq!(server.post("/db/add", &db, &token).await?.0, 201);

    let mut dbs = Vec::<HashMap<&str, &str>>::new();
    dbs.push(db.clone());
    db.insert("name", "db1");
    dbs.push(db);

    let admin_token = server.init_admin().await?;
    let (status, list) = server.get("/admin/db/list", &admin_token).await?;
    assert_eq!(status, 200);
    let mut list: Vec<HashMap<&str, &str>> = serde_json::from_str(&list)?;
    dbs.sort_by(|left, right| left.get("name").unwrap().cmp(right.get("name").unwrap()));
    list.sort_by(|left, right| left.get("name").unwrap().cmp(right.get("name").unwrap()));

    assert_eq!(dbs, list);

    Ok(())
}

#[tokio::test]
async fn user_list() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let bad_token = Some("bad".to_string());
    assert_eq!(server.get("/admin/user/list", &bad_token).await?.0, 403);

    let admin_token = server.init_admin().await?;
    let (status, list) = server.get("/admin/user/list", &admin_token).await?;
    let list: Vec<String> = serde_json::from_str(&list)?;
    assert_eq!(status, 200);
    assert_eq!(list, vec!["admin"]);

    server.init_user("alice", "password123").await?;
    server.init_user("bob", "password456").await?;

    let admin_token = server.init_admin().await?;
    let (status, list) = server.get("/admin/user/list", &admin_token).await?;
    assert_eq!(status, 200);
    let mut list: Vec<String> = serde_json::from_str(&list)?;
    list.sort();
    let expected = vec!["admin", "alice", "bob"];

    assert_eq!(expected, list);

    Ok(())
}
