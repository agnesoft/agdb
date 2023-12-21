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
                &format!("/db/{db}/user/{}/add?db_role=read", other.name),
                &String::new(),
                &user.token
            )
            .await?,
        201
    );
    let mut list = server
        .get::<Vec<DbUser>>(&format!("/db/{db}/user/list"), &other.token)
        .await?
        .1?;
    list.sort();
    let mut expected = vec![
        DbUser {
            user: user.name,
            role: "admin".to_string(),
        },
        DbUser {
            user: other.name,
            role: "read".to_string(),
        },
    ];
    expected.sort();
    assert_eq!(list, expected);
    Ok(())
}

#[tokio::test]
async fn non_user() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    assert_eq!(
        server
            .get::<Vec<DbUser>>(&format!("/db/{db}/user/list"), &other.token)
            .await?
            .0,
        404
    );
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(
        server
            .get::<Vec<DbUser>>("/db/user/db/user/list", NO_TOKEN)
            .await?
            .0,
        401
    );
    Ok(())
}
