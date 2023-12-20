use crate::DbWithRole;
use crate::TestServer;
use crate::ADMIN_DB_REMOVE_URI;
use crate::DB_LIST_URI;
use crate::NO_TOKEN;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct DbName {
    db: String,
}

#[tokio::test]
async fn remove() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let (status, list) = server
        .get::<Vec<DbWithRole>>(DB_LIST_URI, &user.token)
        .await?;
    assert_eq!(status, 200);
    assert!(list?.contains(&DbWithRole {
        name: db.clone(),
        db_type: "mapped".to_string(),
        role: "admin".to_string(),
        size: 2600,
    }));
    let rem = DbName { db: db.clone() };
    assert_eq!(
        server
            .post(ADMIN_DB_REMOVE_URI, &rem, &server.admin_token)
            .await?
            .0,
        204
    );
    let (status, list) = server
        .get::<Vec<DbWithRole>>(DB_LIST_URI, &user.token)
        .await?;
    assert_eq!(status, 200);
    assert!(list?.is_empty());
    assert!(Path::new(&server.data_dir).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = DbName {
        db: format!("{}/some_db", user.name),
    };
    assert_eq!(
        server
            .post(ADMIN_DB_REMOVE_URI, &db, &server.admin_token)
            .await?
            .0,
        466
    );
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let db = DbName {
        db: "missing_user/some_db".to_string(),
    };
    assert_eq!(
        server
            .post(ADMIN_DB_REMOVE_URI, &db, &server.admin_token)
            .await?
            .0,
        466
    );
    Ok(())
}

#[tokio::test]
async fn no_admin_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let db = DbName {
        db: "user/some_db".to_string(),
    };
    assert_eq!(
        server.post(ADMIN_DB_REMOVE_URI, &db, NO_TOKEN).await?.0,
        401
    );
    Ok(())
}
