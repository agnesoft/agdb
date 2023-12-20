use crate::AddUser;
use crate::DbWithRole;
use crate::TestServer;
use crate::ADMIN_DB_USER_ADD_URI;
use crate::DB_LIST_URI;
use crate::NO_TOKEN;

#[tokio::test]
async fn add_db_user() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let role = AddUser {
        database: &db,
        user: &other.name,
        role: "write",
    };
    let (_, list) = server
        .get::<Vec<DbWithRole>>(DB_LIST_URI, &other.token)
        .await?;
    assert_eq!(list?, vec![]);
    assert_eq!(
        server
            .post(ADMIN_DB_USER_ADD_URI, &role, &server.admin_token)
            .await?
            .0,
        201
    );
    let (_, list) = server
        .get::<Vec<DbWithRole>>(DB_LIST_URI, &other.token)
        .await?;
    let expected = vec![DbWithRole {
        name: db,
        db_type: "memory".to_string(),
        role: "write".to_string(),
        size: 2600,
    }];
    assert_eq!(list?, expected);
    Ok(())
}

#[tokio::test]
async fn change_user_role() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let mut role = AddUser {
        database: &db,
        user: &other.name,
        role: "write",
    };
    assert_eq!(
        server
            .post(ADMIN_DB_USER_ADD_URI, &role, &server.admin_token)
            .await?
            .0,
        201
    );
    role.role = "admin";
    assert_eq!(
        server
            .post(ADMIN_DB_USER_ADD_URI, &role, &server.admin_token)
            .await?
            .0,
        201
    );
    role.role = "write";
    role.user = &user.name;
    assert_eq!(
        server
            .post(ADMIN_DB_USER_ADD_URI, &role, &server.admin_token)
            .await?
            .0,
        201
    );

    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let role = AddUser {
        database: "db_not_found",
        user: "some_user",
        role: "read",
    };
    assert_eq!(
        server
            .post(ADMIN_DB_USER_ADD_URI, &role, &server.admin_token)
            .await?
            .0,
        466
    );
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let role = AddUser {
        database: &db,
        user: "user_not_found",
        role: "read",
    };
    assert_eq!(
        server
            .post(ADMIN_DB_USER_ADD_URI, &role, &server.admin_token)
            .await?
            .0,
        464
    );
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
    assert_eq!(
        server.post(ADMIN_DB_USER_ADD_URI, &role, NO_TOKEN).await?.0,
        401
    );
    Ok(())
}
