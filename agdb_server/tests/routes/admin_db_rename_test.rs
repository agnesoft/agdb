use crate::Db;
use crate::TestServer;
use crate::DB_LIST_URI;
use crate::NO_TOKEN;
use std::path::Path;

#[tokio::test]
async fn rename() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let status = server
        .post::<()>(
            &format!("/admin/db/{db}/rename?new_name={}/renamed_db", user.name),
            &None,
            &server.admin_token,
        )
        .await?
        .0;
    assert_eq!(status, 201);
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
        .post::<()>(&format!("/db/{db}/backup"), &None, &user.token)
        .await?;
    let status = server
        .post::<()>(
            &format!("/admin/db/{db}/rename?new_name={}/renamed_db", user.name),
            &None,
            &server.admin_token,
        )
        .await?
        .0;
    assert_eq!(status, 201);
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
        .post::<()>(
            &format!("/admin/db/{db}/rename?new_name={}/renamed_db", other.name),
            &None,
            &server.admin_token,
        )
        .await?
        .0;
    assert_eq!(status, 201);
    let list = server.get::<Vec<Db>>(DB_LIST_URI, &other.token).await?.1?;
    assert_eq!(
        list,
        vec![Db {
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
async fn invalid() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let status = server
        .post::<()>(
            &format!("/admin/db/{db}/rename?new_name={}/a\0a", user.name),
            &None,
            &server.admin_token,
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
    let status = server
        .post::<()>(
            "/admin/db/user/db/rename?new_name=user/renamed_db",
            &None,
            &server.admin_token,
        )
        .await?
        .0;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn target_self() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let status = server
        .post::<()>(
            &format!("/admin/db/{db}/rename?new_name={db}"),
            &None,
            &server.admin_token,
        )
        .await?
        .0;
    assert_eq!(status, 201);
    Ok(())
}

#[tokio::test]
async fn target_exists() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let db2 = server.init_db("mapped", &user).await?;
    let status = server
        .post::<()>(
            &format!("/admin/db/{db}/rename?new_name={db2}"),
            &None,
            &server.admin_token,
        )
        .await?
        .0;
    assert_eq!(status, 467);
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let status = server
        .post::<()>(
            "/admin/db/user/db/rename?new_name=user/renamed_db",
            &None,
            &user.token,
        )
        .await?
        .0;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .post::<()>(
            "/admin/db/user/db/rename?new_name=user/renamed_db",
            &None,
            NO_TOKEN,
        )
        .await?
        .0;
    assert_eq!(status, 401);
    Ok(())
}
