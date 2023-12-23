use crate::DbWithRole;
use crate::TestServer;
use crate::DB_LIST_URI;
use crate::NO_TOKEN;
use serde::Serialize;
use std::path::Path;

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
    let status = server
        .post(
            &format!("/db/{db}/rename?new_name={}/renamed_db", user.name),
            &String::new(),
            &user.token,
        )
        .await?
        .0;
    assert_eq!(status, 204);
    assert!(!Path::new(&server.data_dir).join(db).exists());
    assert!(Path::new(&server.data_dir)
        .join(user.name)
        .join("renamed_db")
        .exists());
    Ok(())
}

#[tokio::test]
async fn rename_with_backup() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    server
        .post(&format!("/db/{db}/backup"), &String::new(), &user.token)
        .await?;
    let status = server
        .post(
            &format!("/db/{db}/rename?new_name={}/renamed_db", user.name),
            &String::new(),
            &user.token,
        )
        .await?
        .0;
    assert_eq!(status, 204);
    assert!(!Path::new(&server.data_dir).join(&db).exists());
    assert!(!Path::new(&server.data_dir)
        .join(&user.name)
        .join("backups")
        .join(format!("{}.bak", db.split_once('/').unwrap().1))
        .exists());
    assert!(Path::new(&server.data_dir)
        .join(&user.name)
        .join("renamed_db")
        .exists());
    assert!(Path::new(&server.data_dir)
        .join(user.name)
        .join("backups")
        .join("renamed_db.bak")
        .exists());
    Ok(())
}

#[tokio::test]
async fn transfer() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let status = server
        .post(
            &format!("/db/{db}/rename?new_name={}/renamed_db", other.name),
            &String::new(),
            &user.token,
        )
        .await?
        .0;
    assert_eq!(status, 204);
    let list = server
        .get::<Vec<DbWithRole>>(DB_LIST_URI, &other.token)
        .await?
        .1?;
    assert_eq!(
        list,
        vec![DbWithRole {
            name: format!("{}/renamed_db", other.name),
            db_type: "mapped".to_string(),
            role: "admin".to_string(),
            size: 2600,
            backup: 0,
        }]
    );
    assert!(!Path::new(&server.data_dir).join(db).exists());
    assert!(Path::new(&server.data_dir)
        .join(other.name)
        .join("renamed_db")
        .exists());
    Ok(())
}

#[tokio::test]
async fn non_owner() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    assert_eq!(
        server
            .put(
                &format!("/db/{db}/user/{}/add?db_role=admin", other.name),
                &String::new(),
                &user.token
            )
            .await?,
        201
    );
    let status = server
        .post(
            &format!("/db/{db}/rename?new_name={}/renamed_db", user.name),
            &String::new(),
            &other.token,
        )
        .await?
        .0;
    assert_eq!(status, 403);
    Ok(())
}

#[tokio::test]
async fn invalid() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let status = server
        .post(
            &format!("/db/{db}/rename?new_name={}/a\0a", user.name),
            &String::new(),
            &user.token,
        )
        .await?
        .0;
    assert_eq!(status, 467);
    assert!(Path::new(&server.data_dir).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let status = server
        .post(
            "/db/user/db/rename?new_name=user/renamed_db",
            &String::new(),
            &user.token,
        )
        .await?
        .0;
    assert_eq!(status, 403);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .post(
            "/db/user/db/rename?new_name=user/renamed_db",
            &String::new(),
            NO_TOKEN,
        )
        .await?
        .0;
    assert_eq!(status, 401);
    Ok(())
}
