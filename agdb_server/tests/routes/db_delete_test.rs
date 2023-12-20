use crate::AddUser;
use crate::DbWithRole;
use crate::TestServer;
use crate::DB_DELETE_URI;
use crate::DB_LIST_URI;
use crate::DB_USER_ADD_URI;
use crate::NO_TOKEN;
use serde::Deserialize;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct DeleteDb {
    db: String,
}

#[tokio::test]
async fn delete() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    assert!(Path::new(&server.data_dir).join(&db).exists());
    let del = DeleteDb { db: db.clone() };
    assert_eq!(server.post(DB_DELETE_URI, &del, &user.token).await?.0, 204);
    let (_, list) = server
        .get::<Vec<DbWithRole>>(DB_LIST_URI, &user.token)
        .await?;
    assert_eq!(list?, vec![]);
    assert!(!Path::new(&server.data_dir).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let del = DeleteDb {
        db: format!("{}/delete_db_not_found", user.name),
    };
    assert_eq!(server.post(DB_DELETE_URI, &del, &user.token).await?.0, 466);
    Ok(())
}

#[tokio::test]
async fn other_user() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let other = server.init_user().await?;
    let del = DeleteDb { db: db.clone() };
    assert_eq!(server.post(DB_DELETE_URI, &del, &other.token).await?.0, 466);
    let (_, list) = server
        .get::<Vec<DbWithRole>>(DB_LIST_URI, &user.token)
        .await?;
    let expected = vec![DbWithRole {
        name: db.clone(),
        db_type: "mapped".to_string(),
        role: "admin".to_string(),
        size: 2600,
    }];
    assert_eq!(list?, expected);
    assert!(Path::new(&server.data_dir).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn with_read_role() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let reader = server.init_user().await?;
    let role = AddUser {
        database: &db,
        user: &reader.name,
        role: "read",
    };
    assert_eq!(
        server.post(DB_USER_ADD_URI, &role, &user.token).await?.0,
        201
    );
    let del = DeleteDb { db: db.clone() };
    assert_eq!(
        server.post(DB_DELETE_URI, &del, &reader.token).await?.0,
        403
    );
    let (_, list) = server
        .get::<Vec<DbWithRole>>(DB_LIST_URI, &user.token)
        .await?;
    let expected = vec![DbWithRole {
        name: db.clone(),
        db_type: "mapped".to_string(),
        role: "admin".to_string(),
        size: 2600,
    }];
    assert_eq!(list?, expected);
    assert!(Path::new(&server.data_dir).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn with_write_role() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let writer = server.init_user().await?;
    let role = AddUser {
        database: &db,
        user: &writer.name,
        role: "write",
    };
    assert_eq!(
        server.post(DB_USER_ADD_URI, &role, &user.token).await?.0,
        201
    );
    let del = DeleteDb { db: db.clone() };
    assert_eq!(
        server.post(DB_DELETE_URI, &del, &writer.token).await?.0,
        403
    );
    let (_, list) = server
        .get::<Vec<DbWithRole>>(DB_LIST_URI, &user.token)
        .await?;
    let expected = vec![DbWithRole {
        name: db.clone(),
        db_type: "mapped".to_string(),
        role: "admin".to_string(),
        size: 2600,
    }];
    assert_eq!(list?, expected);
    assert!(Path::new(&server.data_dir).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn with_admin_role() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let admin = server.init_user().await?;
    let role = AddUser {
        database: &db,
        user: &admin.name,
        role: "admin",
    };
    assert_eq!(
        server.post(DB_USER_ADD_URI, &role, &user.token).await?.0,
        201
    );
    let del = DeleteDb { db: db.clone() };
    assert_eq!(server.post(DB_DELETE_URI, &del, &admin.token).await?.0, 204);
    let (_, list) = server
        .get::<Vec<DbWithRole>>(DB_LIST_URI, &user.token)
        .await?;
    assert_eq!(list?, vec![]);
    assert!(!Path::new(&server.data_dir).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let del = DeleteDb { db: String::new() };
    assert_eq!(server.post(DB_DELETE_URI, &del, NO_TOKEN).await?.0, 401);
    Ok(())
}
