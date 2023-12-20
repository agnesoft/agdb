use std::path::Path;

use crate::AddUser;
use crate::DbWithRole;
use crate::TestServer;
use crate::DB_LIST_URI;
use crate::DB_RENAME_URI;
use crate::DB_USER_ADD_URI;
use crate::NO_TOKEN;
use serde::Serialize;

#[derive(Serialize)]
struct DbRename {
    db: String,
    new_name: String,
}

#[tokio::test]
async fn rename() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let json = DbRename {
        db: db.clone(),
        new_name: format!("{}/renamed_db", &user.name),
    };
    let (status, _response) = server.post(DB_RENAME_URI, &json, &user.token).await?;
    assert_eq!(status, 204);
    let (status, list) = server
        .get::<Vec<DbWithRole>>(DB_LIST_URI, &user.token)
        .await?;
    assert_eq!(status, 200);
    assert_eq!(
        list?,
        vec![DbWithRole {
            name: json.new_name.clone(),
            db_type: "mapped".to_string(),
            role: "admin".to_string(),
            size: 2600,
        }]
    );
    assert!(!Path::new(&server.data_dir).join(db).exists());
    assert!(Path::new(&server.data_dir).join(json.new_name).exists());
    Ok(())
}

#[tokio::test]
async fn rename_non_admin() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let role = AddUser {
        database: &db,
        user: &other.name,
        role: "read",
    };
    let json = DbRename {
        db: db.clone(),
        new_name: format!("{}/renamed_db", &user.name),
    };
    server.post(DB_USER_ADD_URI, &role, &user.token).await?;
    let (status, _response) = server.post(DB_RENAME_URI, &json, &other.token).await?;
    assert_eq!(status, 403);
    Ok(())
}

#[tokio::test]
async fn rename_other_user() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let role = AddUser {
        database: &db,
        user: &other.name,
        role: "admin",
    };
    let json = DbRename {
        db: db.clone(),
        new_name: format!("{}/renamed_db", &other.name),
    };
    server.post(DB_USER_ADD_URI, &role, &user.token).await?;
    let (status, _response) = server.post(DB_RENAME_URI, &json, &other.token).await?;
    assert_eq!(status, 467);
    Ok(())
}

#[tokio::test]
async fn invalid_new_name() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let json = DbRename {
        db: db.clone(),
        new_name: format!("{}/\0??", &user.name),
    };
    let (status, _response) = server.post(DB_RENAME_URI, &json, &user.token).await?;
    assert_eq!(status, 467);
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let json = DbRename {
        db: format!("{}/missing_db", &user.name),
        new_name: format!("{}/renamed_db", &user.name),
    };
    let (status, _response) = server.post(DB_RENAME_URI, &json, &user.token).await?;
    assert_eq!(status, 466);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let json = DbRename {
        db: String::new(),
        new_name: String::new(),
    };
    let (status, _response) = server.post(DB_RENAME_URI, &json, NO_TOKEN).await?;
    assert_eq!(status, 401);
    Ok(())
}
