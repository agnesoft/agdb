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
        user: &other.name,
        role: "write",
    };
    let (_, list) = server
        .get::<Vec<DbWithRole>>(DB_LIST_URI, &other.token)
        .await?;
    assert_eq!(list?, vec![]);
    assert_eq!(
        server
            .put(
                &format!("{ADMIN_DB_USER_ADD_URI}/{db}"),
                &role,
                &server.admin_token
            )
            .await?,
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
        user: &other.name,
        role: "write",
    };
    assert_eq!(
        server
            .put(
                &format!("{ADMIN_DB_USER_ADD_URI}/{db}"),
                &role,
                &server.admin_token
            )
            .await?,
        201
    );
    role.role = "admin";
    assert_eq!(
        server
            .put(
                &format!("{ADMIN_DB_USER_ADD_URI}/{db}"),
                &role,
                &server.admin_token
            )
            .await?,
        201
    );
    role.role = "write";
    role.user = &user.name;
    assert_eq!(
        server
            .put(
                &format!("{ADMIN_DB_USER_ADD_URI}/{db}"),
                &role,
                &server.admin_token
            )
            .await?,
        403
    );

    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let role = AddUser {
        user: "some_user",
        role: "read",
    };
    assert_eq!(
        server
            .put(
                &format!("{ADMIN_DB_USER_ADD_URI}/admin/not_found"),
                &role,
                &server.admin_token
            )
            .await?,
        466
    );
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let role = AddUser {
        user: "user_not_found",
        role: "read",
    };
    assert_eq!(
        server
            .put(
                &format!("{ADMIN_DB_USER_ADD_URI}/user_not_found/db"),
                &role,
                &server.admin_token
            )
            .await?,
        464
    );
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let role = AddUser {
        user: "bob",
        role: "admin",
    };
    assert_eq!(
        server
            .put(
                &format!("{ADMIN_DB_USER_ADD_URI}/user_not_found/db"),
                &role,
                NO_TOKEN
            )
            .await?,
        401
    );
    Ok(())
}
