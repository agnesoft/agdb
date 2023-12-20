use crate::AddUser;
use crate::DbWithRole;
use crate::DbWithSize;
use crate::TestServer;
use crate::DB_LIST_URI;
use crate::DB_OPTIMIZE_URI;
use crate::DB_USER_ADD_URI;
use crate::NO_TOKEN;
use agdb::QueryBuilder;
use serde::Serialize;

#[derive(Serialize)]
struct OptimizeDb {
    db: String,
}

#[derive(Serialize)]
struct DbRename {
    db: String,
    new_name: String,
}

#[tokio::test]
async fn optimize() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    server
        .exec(
            &db,
            &vec![QueryBuilder::insert().nodes().count(100).query().into()],
            &user.token,
        )
        .await?;
    let original_size = server
        .get::<Vec<DbWithRole>>(DB_LIST_URI, &user.token)
        .await?
        .1?[0]
        .size;

    let (status, response) = server
        .post(DB_OPTIMIZE_URI, &OptimizeDb { db: db.clone() }, &user.token)
        .await?;
    assert_eq!(status, 200);
    let optimized_size = serde_json::from_str::<DbWithSize>(&response)?.size;
    assert!(optimized_size < original_size);

    Ok(())
}

#[tokio::test]
async fn read() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let role = AddUser {
        database: &db,
        user: &other.name,
        role: "read",
    };
    server.post(DB_USER_ADD_URI, &role, &user.token).await?;
    let (status, _) = server
        .post(
            DB_OPTIMIZE_URI,
            &OptimizeDb { db: db.clone() },
            &other.token,
        )
        .await?;
    assert_eq!(status, 403);

    Ok(())
}

#[tokio::test]
async fn write() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("mapped", &user).await?;
    let other = server.init_user().await?;
    let role = AddUser {
        database: &db,
        user: &other.name,
        role: "write",
    };
    server.post(DB_USER_ADD_URI, &role, &user.token).await?;
    server
        .exec(
            &db,
            &vec![QueryBuilder::insert().nodes().count(100).query().into()],
            &user.token,
        )
        .await?;
    let original_size = server
        .get::<Vec<DbWithRole>>(DB_LIST_URI, &user.token)
        .await?
        .1?[0]
        .size;
    let (status, response) = server
        .post(
            DB_OPTIMIZE_URI,
            &OptimizeDb { db: db.clone() },
            &other.token,
        )
        .await?;
    assert_eq!(status, 200);
    let optimized_size = serde_json::from_str::<DbWithSize>(&response)?.size;
    assert!(optimized_size < original_size);

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
    let (status, _response) = server.post(DB_OPTIMIZE_URI, &json, &user.token).await?;
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
    let (status, _response) = server.post(DB_OPTIMIZE_URI, &json, NO_TOKEN).await?;
    assert_eq!(status, 401);
    Ok(())
}
