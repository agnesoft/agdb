use crate::TestServer;
use crate::ADMIN;
use agdb_api::DbType;
use agdb_api::DbUser;
use agdb_api::DbUserRole;

#[tokio::test]
async fn list() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let user = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    let (status, list) = server.api.admin_db_user_list(owner, db).await?;
    assert_eq!(status, 200);
    assert_eq!(
        list,
        vec![
            DbUser {
                user: owner.to_string(),
                role: DbUserRole::Admin,
            },
            DbUser {
                user: user.to_string(),
                role: DbUserRole::Write,
            }
        ]
    );
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_db_user_list("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);

    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .admin_db_user_list("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_db_user_list("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}
