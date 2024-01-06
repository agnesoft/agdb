use crate::DbUser;
use crate::TestServer;
use crate::NO_TOKEN;
use agdb_api::DbType;

#[tokio::test]
async fn remove() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let other = server.init_user().await?;
    let db = server.init_db(DbType::Memory, &user).await?;
    assert_eq!(
        server
            .put::<()>(
                &format!("/admin/db/{db}/user/{}/add?db_role=write", other.name),
                &None,
                &server.admin_token
            )
            .await?,
        201
    );
    assert_eq!(
        server
            .delete(
                &format!("/admin/db/{db}/user/{}/remove", other.name),
                &server.admin_token
            )
            .await?,
        204
    );
    let list = server
        .get::<Vec<DbUser>>(&format!("/admin/db/{db}/user/list"), &server.admin_token)
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
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db(DbType::Memory, &user).await?;
    assert_eq!(
        server
            .delete(
                &format!("/admin/db/{db}/user/{}/remove", user.name),
                &server.admin_token
            )
            .await?,
        403
    );
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    assert_eq!(
        server
            .delete("/admin/db/user/db/user/another/remove", &server.admin_token)
            .await?,
        404
    );
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db(DbType::Memory, &user).await?;
    assert_eq!(
        server
            .delete(
                &format!("/admin/db/{db}/user/other/remove"),
                &server.admin_token
            )
            .await?,
        404
    );
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = server.init_user().await?;
    assert_eq!(
        server
            .delete("/admin/db/user/db/user/user/remove", &user.token)
            .await?,
        401
    );
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    assert_eq!(
        server
            .delete("/admin/db/user/db/user/user/remove", NO_TOKEN)
            .await?,
        401
    );
    Ok(())
}
