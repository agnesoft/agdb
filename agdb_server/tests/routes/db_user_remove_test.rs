use crate::AddUser;
use crate::Db;
use crate::TestServer;
use crate::DB_LIST_URI;
use crate::DB_USER_ADD_URI;
use crate::DB_USER_REMOVE_URI;
use crate::NO_TOKEN;
use serde::Serialize;

#[derive(Serialize)]
struct RemoveUser {
    database: String,
    user: String,
}

#[tokio::test]
async fn remove() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let reader = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let role = AddUser {
        database: &db,
        user: &reader.name,
        role: "read",
    };
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &reader.token).await?;
    assert_eq!(list?, vec![]);
    assert_eq!(
        server.post(DB_USER_ADD_URI, &role, &user.token).await?.0,
        201
    );
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &reader.token).await?;
    let expected = vec![Db {
        name: db.clone(),
        db_type: "memory".to_string(),
    }];
    assert_eq!(list?, expected);
    let rem = RemoveUser {
        database: db,
        user: reader.name,
    };
    assert_eq!(
        server.post(DB_USER_REMOVE_URI, &rem, &user.token).await?.0,
        204
    );
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &reader.token).await?;
    assert_eq!(list?, vec![]);
    Ok(())
}

#[tokio::test]
async fn remove_user_non_admin() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let role = AddUser {
        database: &db,
        user: &other.name,
        role: "read",
    };
    assert_eq!(
        server.post(DB_USER_ADD_URI, &role, &user.token).await?.0,
        201
    );
    let rem = RemoveUser {
        database: db,
        user: user.name,
    };
    assert_eq!(
        server.post(DB_USER_REMOVE_URI, &rem, &other.token).await?.0,
        403
    );
    Ok(())
}

#[tokio::test]
async fn remove_self() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let role = AddUser {
        database: &db,
        user: &other.name,
        role: "read",
    };
    assert_eq!(
        server.post(DB_USER_ADD_URI, &role, &user.token).await?.0,
        201
    );
    let rem = RemoveUser {
        database: db,
        user: other.name,
    };
    assert_eq!(
        server.post(DB_USER_REMOVE_URI, &rem, &other.token).await?.0,
        204
    );
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &other.token).await?;
    assert_eq!(list?, vec![]);
    Ok(())
}

#[tokio::test]
async fn remove_self_admin() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let role = AddUser {
        database: &db,
        user: &other.name,
        role: "admin",
    };
    assert_eq!(
        server.post(DB_USER_ADD_URI, &role, &user.token).await?.0,
        201
    );
    let rem = RemoveUser {
        database: db,
        user: other.name,
    };
    assert_eq!(
        server.post(DB_USER_REMOVE_URI, &rem, &other.token).await?.0,
        204
    );
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &other.token).await?;
    assert_eq!(list?, vec![]);
    Ok(())
}

#[tokio::test]
async fn remove_self_admin_last() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let rem = RemoveUser {
        database: db.clone(),
        user: user.name,
    };
    assert_eq!(
        server.post(DB_USER_REMOVE_URI, &rem, &user.token).await?.0,
        403
    );
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &user.token).await?;
    let expected = vec![Db {
        name: db,
        db_type: "memory".to_string(),
    }];
    assert_eq!(list?, expected);
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let rem = RemoveUser {
        database: format!("{}/db_not_found", user.name),
        user: String::new(),
    };
    assert_eq!(
        server.post(DB_USER_REMOVE_URI, &rem, &user.token).await?.0,
        466
    );
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let rem = RemoveUser {
        database: db,
        user: "user_not_found".to_string(),
    };
    assert_eq!(
        server.post(DB_USER_REMOVE_URI, &rem, &user.token).await?.0,
        464
    );
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let rem = RemoveUser {
        database: String::new(),
        user: String::new(),
    };
    assert_eq!(
        server.post(DB_USER_REMOVE_URI, &rem, NO_TOKEN).await?.0,
        401
    );
    Ok(())
}
