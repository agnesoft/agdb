use crate::DbWithRole;
use crate::TestServer;
use crate::DB_LIST_URI;
use crate::NO_TOKEN;

#[tokio::test]
async fn add_db_user() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    assert_eq!(
        server
            .put(
                &format!("/admin/db/{db}/user/{}/add?db_role=write", other.name),
                &String::new(),
                &server.admin_token
            )
            .await?,
        201
    );
    let list = server
        .get::<Vec<DbWithRole>>(DB_LIST_URI, &other.token)
        .await?
        .1;
    assert_eq!(
        list?,
        vec![DbWithRole {
            name: db,
            db_type: "memory".to_string(),
            role: "write".to_string(),
            size: 2600,
            backup: 0,
        }]
    );
    Ok(())
}

#[tokio::test]
async fn change_user_role() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    assert_eq!(
        server
            .put(
                &format!("/admin/db/{db}/user/{}/add?db_role=write", other.name),
                &String::new(),
                &server.admin_token
            )
            .await?,
        201
    );
    assert_eq!(
        server
            .put(
                &format!("/admin/db/{db}/user/{}/add?db_role=admin", other.name),
                &String::new(),
                &server.admin_token
            )
            .await?,
        201
    );
    let list = server
        .get::<Vec<DbWithRole>>(DB_LIST_URI, &other.token)
        .await?
        .1;
    assert_eq!(
        list?,
        vec![DbWithRole {
            name: db,
            db_type: "memory".to_string(),
            role: "admin".to_string(),
            size: 2600,
            backup: 0,
        }]
    );
    Ok(())
}

#[tokio::test]
async fn change_owner_role() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    assert_eq!(
        server
            .put(
                &format!("/admin/db/{db}/user/{}/add?db_role=admin", other.name),
                &String::new(),
                &server.admin_token
            )
            .await?,
        201
    );
    assert_eq!(
        server
            .put(
                &format!("/admin/db/{db}/user/{}/add?db_role=write", user.name),
                &String::new(),
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
    assert_eq!(
        server
            .put(
                "/admin/db/user/db/user/other/add?db_role=admin",
                &String::new(),
                &server.admin_token
            )
            .await?,
        404
    );
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    assert_eq!(
        server
            .put(
                &format!("/admin/db/{db}/user/other/add?db_role=admin"),
                &String::new(),
                &server.admin_token
            )
            .await?,
        404
    );
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    assert_eq!(
        server
            .put(
                "/admin/db/user/db/user/other/add?db_role=admin",
                &String::new(),
                &user.token
            )
            .await?,
        401
    );
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(
        server
            .put(
                "/admin/db/user/db/user/other/add?db_role=admin",
                &String::new(),
                NO_TOKEN
            )
            .await?,
        401
    );
    Ok(())
}
