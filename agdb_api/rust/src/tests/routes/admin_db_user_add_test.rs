use crate::DbKind;
use crate::DbUserRole;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn db_user_add() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let status = server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    assert_eq!(status, 201);
    server.api.user_login(user, user).await?;
    let list = server.api.db_list().await?.1;
    assert_eq!(list.len(), 1, "{list:?}");
    assert!(
        list.iter().any(|db_entry| {
            matches!(
                (&db_entry.db, &db_entry.owner, db_entry.db_type, db_entry.role),
                (listed_db, listed_owner, DbKind::Mapped, DbUserRole::Write)
                    if listed_db == db && listed_owner == owner
            )
        }),
        "{list:?}"
    );
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn change_user_role() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let status = server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    assert_eq!(status, 201);
    let status = server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Read)
        .await?;
    assert_eq!(status, 201);
    server.api.user_login(user, user).await?;
    let list = server.api.db_list().await?.1;
    assert_eq!(list.len(), 1, "{list:?}");
    assert!(
        list.iter().any(|db_entry| {
            matches!(
                (&db_entry.db, &db_entry.owner, db_entry.db_type, db_entry.role),
                (listed_db, listed_owner, DbKind::Mapped, DbUserRole::Read)
                    if listed_db == db && listed_owner == owner
            )
        }),
        "{list:?}"
    );
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn change_owner_role() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let status = server
        .api
        .admin_db_user_add(owner, db, owner, DbUserRole::Write)
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
    let status = server
        .api
        .admin_db_user_add(owner, "db", user, DbUserRole::Write)
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
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let status = server
        .api
        .admin_db_user_add(owner, db, "user", DbUserRole::Write)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .admin_db_user_add(owner, "db", "user", DbUserRole::Write)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_db_user_add("owner", "db", "user", DbUserRole::Write)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __db_user_add_type_def(),
        __change_user_role_type_def(),
        __change_owner_role_type_def(),
        __db_not_found_type_def(),
        __user_not_found_type_def(),
        __non_admin_type_def(),
        __no_token_type_def(),
    ]
}
