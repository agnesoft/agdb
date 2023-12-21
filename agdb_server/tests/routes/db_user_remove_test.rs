use crate::DbUser;
use crate::DbWithRole;
use crate::TestServer;
use crate::DB_LIST_URI;
use crate::NO_TOKEN;

#[tokio::test]
async fn remove() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    assert_eq!(
        server
            .put(
                &format!("/db/{db}/user/{}/add?db_role=write", other.name),
                &String::new(),
                &user.token
            )
            .await?,
        201
    );
    assert_eq!(
        server
            .delete(&format!("/db/{db}/user/{}/remove", other.name), &user.token)
            .await?,
        204
    );
    let list = server
        .get::<Vec<DbUser>>(&format!("/db/{db}/user/list"), &user.token)
        .await?
        .1;
    assert_eq!(
        list?,
        vec![DbUser {
            user: user.name.clone(),
            role: "admin".to_string(),
        },]
    );
    Ok(())
}

#[tokio::test]
async fn remove_owner() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    assert_eq!(
        server
            .put(
                &format!("/db/{db}/user/{}/add?db_role=admin", other.name),
                &String::new(),
                &user.token
            )
            .await?,
        201
    );
    assert_eq!(
        server
            .delete(&format!("/db/{db}/user/{}/remove", user.name), &other.token)
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
            .put(
                &format!("/db/{db}/user/{}/add?db_role=write", other.name),
                &String::new(),
                &user.token
            )
            .await?,
        201
    );
    assert_eq!(
        server
            .put(
                &format!("/db/{db}/user/{}/add?db_role=write", another.name),
                &String::new(),
                &user.token
            )
            .await?,
        201
    );
    assert_eq!(
        server
            .delete(
                &format!("/db/{db}/user/{}/remove", other.name),
                &another.token
            )
            .await?,
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
    assert_eq!(
        server
            .put(
                &format!("/db/{db}/user/{}/add?db_role=write", other.name),
                &String::new(),
                &user.token
            )
            .await?,
        201
    );
    assert_eq!(
        server
            .delete(
                &format!("/db/{db}/user/{}/remove", other.name),
                &other.token
            )
            .await?,
        204
    );
    let list = server
        .get::<Vec<DbWithRole>>(DB_LIST_URI, &other.token)
        .await?
        .1;
    assert_eq!(list?, vec![]);
    Ok(())
}

#[tokio::test]
async fn remove_self_owner() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;
    assert_eq!(
        server
            .put(
                &format!("/db/{db}/user/{}/add?db_role=admin", other.name),
                &String::new(),
                &user.token
            )
            .await?,
        201
    );
    assert_eq!(
        server
            .delete(&format!("/db/{db}/user/{}/remove", user.name), &user.token)
            .await?,
        403
    );
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    assert_eq!(
        server
            .delete(
                &format!("/db/{}/db/user/user/remove", user.name),
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
            .delete(&format!("/db/{db}/user/other/remove"), &user.token)
            .await?,
        404
    );
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    assert_eq!(
        server
            .delete("/db/user/db/user/user/remove", NO_TOKEN)
            .await?,
        401
    );
    Ok(())
}
