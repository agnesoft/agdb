use crate::AddUser;
use crate::TestServer;
use crate::ADMIN_DB_USER_LIST_URI;
use crate::DB_USER_ADD_URI;
use crate::NO_TOKEN;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct DbUser {
    database: String,
    user: String,
    role: String,
}

#[tokio::test]
async fn list_users() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    let role = AddUser {
        user: &other.name,
        role: "read",
    };
    assert_eq!(
        server.post(DB_USER_ADD_URI, &role, &user.token).await?.0,
        201
    );
    let (_, list) = server
        .get::<Vec<DbUser>>(
            &format!("{ADMIN_DB_USER_LIST_URI}?db={}", &db),
            &server.admin_token,
        )
        .await?;
    let mut list = list?;
    list.sort();
    let mut expected = vec![
        DbUser {
            database: db.clone(),
            user: user.name,
            role: "admin".to_string(),
        },
        DbUser {
            database: db,
            user: other.name,
            role: "read".to_string(),
        },
    ];
    expected.sort();
    assert_eq!(list, expected);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(
        server
            .get::<Vec<DbUser>>(&format!("{ADMIN_DB_USER_LIST_URI}?db=some_db"), NO_TOKEN)
            .await?
            .0,
        401
    );
    Ok(())
}
