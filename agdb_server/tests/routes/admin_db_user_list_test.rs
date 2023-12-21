use crate::TestServer;
use crate::NO_TOKEN;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct DbUser {
    user: String,
    role: String,
}

#[tokio::test]
async fn list() -> anyhow::Result<()> {
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
        .get::<Vec<DbUser>>(&format!("/admin/db/{db}/user/list"), &server.admin_token)
        .await?
        .1;
    assert_eq!(
        list?,
        vec![
            DbUser {
                user: user.name.clone(),
                role: "admin".to_string(),
            },
            DbUser {
                user: other.name.clone(),
                role: "write".to_string(),
            }
        ]
    );
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(
        server
            .get::<Vec<DbUser>>("/admin/db/user/db/user/list", &server.admin_token)
            .await?
            .0,
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
            .get::<Vec<DbUser>>("/admin/db/user/db/user/list", &user.token)
            .await?
            .0,
        401
    );
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(
        server
            .get::<Vec<DbUser>>("/admin/db/user/db/user/list", NO_TOKEN)
            .await?
            .0,
        401
    );
    Ok(())
}
