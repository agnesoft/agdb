use std::path::Path;

use crate::Db;
use crate::TestServer;
use crate::ADMIN_DB_REMOVE_URI;
use crate::DB_LIST_URI;
use crate::NO_TOKEN;
use serde::Serialize;

#[derive(Serialize)]
struct DbName {
    name: String,
}

#[tokio::test]
async fn remove() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &user.token).await?;
    assert_eq!(status, 200);
    assert!(list?.contains(&Db {
        name: db.clone(),
        db_type: "mapped".to_string(),
    }));
    let rem = DbName { name: db.clone() };
    assert_eq!(
        server
            .post(ADMIN_DB_REMOVE_URI, &rem, &server.admin_token)
            .await?
            .0,
        204
    );
    let (status, list) = server.get::<Vec<Db>>(DB_LIST_URI, &user.token).await?;
    assert_eq!(status, 200);
    assert!(list?.is_empty());
    assert!(Path::new(&server.dir).join(db).exists());
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = DbName {
        name: format!("{}/some_db", user.name),
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
        name: "missing_user/some_db".to_string(),
    };
    assert_eq!(
        server
            .post(ADMIN_DB_REMOVE_URI, &db, &server.admin_token)
            .await?
            .0,
        464
    );
    Ok(())
}

#[tokio::test]
async fn no_admin_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let db = DbName {
        name: "user/some_db".to_string(),
    };
    assert_eq!(
        server.post(ADMIN_DB_REMOVE_URI, &db, NO_TOKEN).await?.0,
        401
    );
    Ok(())
}
