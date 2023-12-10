use crate::framework::AddUser;
use crate::framework::Db;
use crate::framework::TestServer;
use crate::framework::DB_LIST_URI;
use crate::framework::DB_USER_ADD_URI;
use crate::framework::NO_TOKEN;

#[tokio::test]
async fn add_reader() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let (reader_name, reader_token) = server.init_user().await?;
    let db = server.init_db("memory", &token).await?;
    let role = AddUser {
        database: &db,
        user: &reader_name,
        role: "read",
    };
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &reader_token).await?;
    assert_eq!(list?, vec![]);
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 201);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &reader_token).await?;
    let expected = vec![Db {
        name: db,
        db_type: "memory".to_string(),
    }];
    assert_eq!(list?, expected);
    Ok(())
}

#[tokio::test]
async fn add_writer() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let (writer_name, writer_token) = server.init_user().await?;
    let db = server.init_db("memory", &token).await?;
    let role = AddUser {
        database: &db,
        user: &writer_name,
        role: "write",
    };
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &writer_token).await?;
    assert_eq!(list?, vec![]);
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 201);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &writer_token).await?;
    let expected = vec![Db {
        name: db,
        db_type: "memory".to_string(),
    }];
    assert_eq!(list?, expected);
    Ok(())
}

#[tokio::test]
async fn add_admin() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let (admin_name, admin_token) = server.init_user().await?;
    let db = server.init_db("memory", &token).await?;
    let role = AddUser {
        database: &db,
        user: &admin_name,
        role: "admin",
    };
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &admin_token).await?;
    assert_eq!(list?, vec![]);
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 201);
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &admin_token).await?;
    let expected = vec![Db {
        name: db,
        db_type: "memory".to_string(),
    }];
    assert_eq!(list?, expected);
    Ok(())
}

#[tokio::test]
async fn add_self() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (name, token) = server.init_user().await?;
    let db = server.init_db("memory", &token).await?;
    let role = AddUser {
        database: &db,
        user: &name,
        role: "read",
    };
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 403);
    Ok(())
}

#[tokio::test]
async fn add_admin_as_non_admin() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let (_, writer_token) = server.init_user().await?;
    let (other_name, other_token) = server.init_user().await?;
    let db = server.init_db("memory", &token).await?;
    let role = AddUser {
        database: &db,
        user: &other_name,
        role: "write",
    };
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &other_token).await?;
    assert_eq!(list?, vec![]);
    assert_eq!(
        server.post(DB_USER_ADD_URI, &role, &writer_token).await?.0,
        403
    );
    let (_, list) = server.get::<Vec<Db>>(DB_LIST_URI, &other_token).await?;
    assert_eq!(list?, vec![]);
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let role = AddUser {
        database: "db_not_found",
        user: "some_user",
        role: "read",
    };
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 466);
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let (_, token) = server.init_user().await?;
    let db = server.init_db("memory", &token).await?;
    let role = AddUser {
        database: &db,
        user: "user_not_found",
        role: "read",
    };
    assert_eq!(server.post(DB_USER_ADD_URI, &role, &token).await?.0, 464);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let role = AddUser {
        database: "my_db",
        user: "bob",
        role: "admin",
    };
    assert_eq!(server.post(DB_USER_ADD_URI, &role, NO_TOKEN).await?.0, 401);
    Ok(())
}
