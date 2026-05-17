use crate::DbKind;
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
    let db1 = &next_db_name();
    let db2 = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db1, DbKind::Memory).await?;
    server.api.admin_db_add(user, db2, DbKind::Memory).await?;
    server
        .api
        .admin_db_user_add(owner, db1, user, DbUserRole::Read)
        .await?;
    server.api.user_login(user, user).await?;
    let (status, list) = server.api.db_list().await?;
    assert_eq!(status, 200);
    assert_eq!(list.len(), 2, "{list:?}");
    assert!(
        list.iter().any(|db_entry| {
            matches!(
                (&db_entry.db, &db_entry.owner, db_entry.db_type, db_entry.role),
                (listed_db, listed_owner, DbKind::Memory, DbUserRole::Read)
                    if listed_db == db1 && listed_owner == owner
            )
        }),
        "{list:?}"
    );
    assert!(
        list.iter().any(|db_entry| {
            matches!(
                (&db_entry.db, &db_entry.owner, db_entry.db_type, db_entry.role),
                (listed_db, listed_owner, DbKind::Memory, DbUserRole::Admin)
                    if listed_db == db2 && listed_owner == user
            )
        }),
        "{list:?}"
    );
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn with_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    server.api.db_backup(owner, db).await?;
    let (status, list) = server.api.db_list().await?;
    assert_eq!(status, 200);
    let db = list
        .iter()
        .find(|d| d.db == *db && d.owner == *owner)
        .unwrap();
    assert_ne!(db.backup, 0);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn list_empty() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let (status, list) = server.api.db_list().await?;
    assert_eq!(status, 200);
    assert!(list.is_empty());
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn list_no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server.api.db_list().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __list_type_def(),
        __with_backup_type_def(),
        __list_empty_type_def(),
        __list_no_token_type_def(),
    ]
}
