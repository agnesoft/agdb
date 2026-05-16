use crate::DbKind;
use crate::DbUserRole;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use std::path::Path;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn delete() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.user_login(owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    assert!(Path::new(&server.data_dir).join(owner).join(db).exists());
    let status = server.api.db_delete(owner, db).await?;
    assert_eq!(status, 204);
    assert!(!Path::new(&server.data_dir).join(owner).join(db).exists());
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn delete_in_memory() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.user_login(owner).await?;
    server.api.db_add(owner, db, DbKind::Memory).await?;
    let status = server.api.db_delete(owner, db).await?;
    assert_eq!(status, 204);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn delete_with_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db_path = Path::new(&server.data_dir).join(owner).join(db);
    let db_backup_path = Path::new(&server.data_dir)
        .join(owner)
        .join("backups")
        .join(format!("{db}.bak"));
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.user_login(owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    server.api.db_backup(owner, db).await?;
    assert!(db_path.exists());
    assert!(db_backup_path.exists());
    let status = server.api.db_delete(owner, db).await?;
    assert_eq!(status, 204);
    assert!(!db_path.exists());
    assert!(!db_backup_path.exists());
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn delete_in_memory_with_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db_path = Path::new(&server.data_dir).join(owner).join(db);
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.user_login(owner).await?;
    server.api.db_add(owner, db, DbKind::Memory).await?;
    assert!(!db_path.exists());
    server.api.db_backup(owner, db).await?;
    assert!(db_path.exists());
    let status = server.api.db_delete(owner, db).await?;
    assert!(!db_path.exists());
    assert_eq!(status, 204);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn db_not_found() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.user_login(owner).await?;
    let status = server.api.db_delete(owner, "db").await.unwrap_err().status;
    assert_eq!(status, 404);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn non_owner() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Admin)
        .await?;
    server.user_login(user).await?;
    let status = server.api.db_delete(owner, db).await.unwrap_err().status;
    assert_eq!(status, 403);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .db_delete("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __delete_type_def(),
        __delete_in_memory_type_def(),
        __delete_with_backup_type_def(),
        __delete_in_memory_with_backup_type_def(),
        __db_not_found_type_def(),
        __non_owner_type_def(),
        __no_token_type_def(),
    ]
}
