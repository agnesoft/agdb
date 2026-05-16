use crate::DbKind;
use crate::DbUser;
use crate::DbUserRole;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn list() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.user_login(owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    server
        .api
        .db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    let (status, mut list) = server.api.db_user_list(owner, db).await?;
    list.sort();
    let mut expected = vec![
        DbUser {
            username: owner.to_string(),
            role: DbUserRole::Admin,
        },
        DbUser {
            username: user.to_string(),
            role: DbUserRole::Write,
        },
    ];
    expected.sort();

    assert_eq!(status, 200);
    assert_eq!(list, expected);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn non_db_user() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbKind::Memory).await?;
    server.user_login(user).await?;
    let status = server.api.db_user_list(owner, db).await.unwrap_err().status;
    assert_eq!(status, 404);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .db_user_list("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __list_type_def(),
        __non_db_user_type_def(),
        __no_token_type_def(),
    ]
}
