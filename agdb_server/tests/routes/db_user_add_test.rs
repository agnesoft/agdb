use crate::Db;
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
            .put::<()>(
                &format!("/db/{db}/user/{}/add?db_role=write", other.name),
                &None,
                &user.token
            )
            .await?,
        201
    );
    let list = server.get::<Vec<Db>>(DB_LIST_URI, &other.token).await?.1;
    assert_eq!(
        list?,
        vec![Db {
            name: db,
            db_type: "memory".to_string(),
            role: "write".to_string(),
            size: 2632,
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
            .put::<()>(
                &format!("/db/{db}/user/{}/add?db_role=admin", other.name),
                &None,
                &user.token
            )
            .await?,
        201
    );
    assert_eq!(
        server
            .put::<()>(
                &format!("/db/{db}/user/{}/add?db_role=read", other.name),
                &None,
                &user.token
            )
            .await?,
        201
    );
    let list = server.get::<Vec<Db>>(DB_LIST_URI, &other.token).await?.1;
    assert_eq!(
        list?,
        vec![Db {
            name: db,
            db_type: "memory".to_string(),
            role: "read".to_string(),
            size: 2632,
            backup: 0,
        }]
    );
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    assert_eq!(
        server
            .put::<()>(
                "/db/user/db/user/other/add?db_role=admin",
                &None,
                &user.token
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
            .put::<()>(
                &format!("/db/{db}/user/other/add?db_role=admin"),
                &None,
                &user.token
            )
            .await?,
        404
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
            .put::<()>(
                &format!("/db/{db}/user/{}/add?db_role=admin", other.name),
                &None,
                &user.token
            )
            .await?,
        201
    );
    assert_eq!(
        server
            .put::<()>(
                &format!("/db/{db}/user/{}/add?db_role=read", user.name),
                &None,
                &other.token
            )
            .await?,
        403
    );
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let another = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    assert_eq!(
        server
            .put::<()>(
                &format!("/db/{db}/user/{}/add?db_role=write", other.name),
                &None,
                &user.token
            )
            .await?,
        201
    );
    assert_eq!(
        server
            .put::<()>(
                &format!("/db/{db}/user/{}/add?db_role=admin", another.name),
                &None,
                &other.token
            )
            .await?,
        403
    );
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(
        server
            .put::<()>("/db/user/db/user/other/add?db_role=admin", &None, NO_TOKEN)
            .await?,
        401
    );
    Ok(())
}
