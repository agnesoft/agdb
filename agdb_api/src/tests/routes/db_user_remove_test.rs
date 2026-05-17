use crate::DbKind;
use crate::DbUserRole;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn remove() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    server
        .api
        .db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    let list = server.api.db_user_list(owner, db).await?.1;
    assert_eq!(list.len(), 2);
    let status = server.api.db_user_remove(owner, db, user).await?;
    assert_eq!(status, 204);
    let list = server.api.db_user_list(owner, db).await?.1;
    assert_eq!(list.len(), 1);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn remove_owner() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Admin)
        .await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .db_user_remove(owner, db, owner)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let other = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_user_add(other, other).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    server
        .api
        .admin_db_user_add(owner, db, other, DbUserRole::Write)
        .await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .db_user_remove(owner, db, other)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn remove_self() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Read)
        .await?;
    server.api.user_login(user, user).await?;
    let status = server.api.db_user_remove(owner, db, user).await?;
    assert_eq!(status, 204);
    let list = server.api.db_list().await?.1;
    assert!(list.is_empty());
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn remove_self_owner() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .db_user_remove(owner, db, owner)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn db_not_found() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .db_user_remove(owner, "db", user)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn user_not_found() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let status = server
        .api
        .db_user_remove(owner, db, "user")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .db_user_remove("owner", "db", "user")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __remove_type_def(),
        __remove_owner_type_def(),
        __non_admin_type_def(),
        __remove_self_type_def(),
        __remove_self_owner_type_def(),
        __db_not_found_type_def(),
        __user_not_found_type_def(),
        __no_token_type_def(),
    ]
}
