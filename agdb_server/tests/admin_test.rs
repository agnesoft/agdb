pub mod framework;

use crate::framework::Db;
use crate::framework::TestServer;
use crate::framework::User;
use crate::framework::UserStatus;
use crate::framework::ADMIN_CHANGE_PASSWORD_URI;
use crate::framework::ADMIN_DB_LIST_URI;
use crate::framework::ADMIN_USER_LIST_URI;
use crate::framework::CREATE_USER_URI;
use crate::framework::DB_ADD_URI;
use crate::framework::LOGIN_URI;
use crate::framework::NO_TOKEN;

#[tokio::test]
async fn create_user() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let admin = server.init_admin().await?;
    let bad = Some("bad".to_string());
    let user = User {
        name: "alice",
        password: "password123",
    };
    let short_name = User {
        name: "a",
        password: "password123",
    };
    let short_pswd = User {
        name: "alice",
        password: "pswd",
    };

    assert_eq!(server.post(CREATE_USER_URI, &user, NO_TOKEN).await?.0, 401); //permission denied
    assert_eq!(server.post(CREATE_USER_URI, &user, &bad).await?.0, 403); //forbidden
    assert_eq!(
        server.post(CREATE_USER_URI, &short_name, &admin).await?.0,
        461
    ); //short name
    assert_eq!(
        server.post(CREATE_USER_URI, &short_pswd, &admin).await?.0,
        462
    ); //short password
    assert_eq!(server.post(CREATE_USER_URI, &user, &admin).await?.0, 201); //created
    assert_eq!(server.post(CREATE_USER_URI, &user, &admin).await?.0, 463); //user exists
    Ok(())
}

#[tokio::test]
async fn change_password() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let admin = server.init_admin().await?;
    let bad = Some("bad".to_string());
    let user = User {
        name: "alice",
        password: "password123",
    };
    let short = User {
        name: "alice",
        password: "pswd",
    };
    let change = User {
        name: "alice",
        password: "mypassword456",
    };
    let new_user = User {
        name: "alice",
        password: "mypassword456",
    };
    let unknown_user = User {
        name: "bob",
        password: "mypassword456",
    };

    assert_eq!(server.post(CREATE_USER_URI, &user, &admin).await?.0, 201); //created
    assert_eq!(
        server
            .post(ADMIN_CHANGE_PASSWORD_URI, &change, &bad)
            .await?
            .0,
        403
    ); //forbidden
    assert_eq!(
        server
            .post(ADMIN_CHANGE_PASSWORD_URI, &unknown_user, &admin)
            .await?
            .0,
        403
    ); //user not found
    assert_eq!(
        server
            .post(ADMIN_CHANGE_PASSWORD_URI, &short, &admin)
            .await?
            .0,
        462
    ); //short password
    assert_eq!(
        server
            .post(ADMIN_CHANGE_PASSWORD_URI, &change, &admin)
            .await?
            .0,
        200
    ); //ok
    assert_eq!(server.post(LOGIN_URI, &new_user, NO_TOKEN).await?.0, 200);

    Ok(())
}

#[tokio::test]
async fn db_list() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let token1 = server.init_user("alice", "password123").await?;
    let token2 = server.init_user("bob", "password456").await?;
    let admin = server.init_admin().await?;
    let bad = Some("bad".to_string());
    let db1 = Db {
        name: "db1".to_string(),
        db_type: "mapped".to_string(),
    };
    let db2 = Db {
        name: "db2".to_string(),
        db_type: "mapped".to_string(),
    };

    assert_eq!(server.get::<()>(ADMIN_DB_LIST_URI, &bad).await?.0, 403);
    let (status, list) = server.get::<Vec<Db>>(ADMIN_DB_LIST_URI, &admin).await?;
    assert_eq!(status, 200);
    assert!(list?.is_empty());

    assert_eq!(server.post(DB_ADD_URI, &db1, &token1).await?.0, 201);
    assert_eq!(server.post(DB_ADD_URI, &db2, &token2).await?.0, 201);

    let (status, list) = server.get::<Vec<Db>>(ADMIN_DB_LIST_URI, &admin).await?;
    let mut list = list?;
    list.sort();
    assert_eq!(status, 200);
    assert_eq!(list, vec![db1, db2]);

    Ok(())
}

#[tokio::test]
async fn user_list() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.init_user("alice", "password123").await?;
    server.init_user("bob", "password456").await?;
    let admin = server.init_admin().await?;
    let bad = Some("bad".to_string());
    let admin_user = UserStatus {
        name: "admin".to_string(),
    };
    let user1 = UserStatus {
        name: "alice".to_string(),
    };
    let user2 = UserStatus {
        name: "bob".to_string(),
    };

    assert_eq!(server.get::<()>(ADMIN_USER_LIST_URI, &bad).await?.0, 403); //forbidden
    let (status, list) = server
        .get::<Vec<UserStatus>>(ADMIN_USER_LIST_URI, &admin)
        .await?;
    let mut list = list?;
    list.sort();
    assert_eq!(status, 200); //Ok
    assert_eq!(list, vec![admin_user, user1, user2]);

    Ok(())
}
