use crate::DbKind;
use crate::DbResource;
use crate::DbUserRole;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb::QueryBuilder;
use std::path::Path;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn clear_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.user_login(owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let queries = &[QueryBuilder::insert().nodes().count(1).query().into()];
    server.api.db_exec_mut(owner, db, queries).await?;
    server.api.db_backup(owner, db).await?;
    let (status, db) = server.api.db_clear(owner, db, DbResource::Backup).await?;
    assert_eq!(status, 200);
    assert_eq!(db.backup, 0);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn clear_audit() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.user_login(owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let queries = &[QueryBuilder::insert().nodes().count(1).query().into()];
    server.api.db_exec_mut(owner, db, queries).await?;
    let (status, _) = server.api.db_clear(owner, db, DbResource::Audit).await?;
    assert_eq!(status, 200);
    let db_audit_file = Path::new(&server.data_dir)
        .join(owner)
        .join("audit")
        .join(format!("{db}.log"));
    assert!(!db_audit_file.exists());
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn clear_db() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.user_login(owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let queries = &[QueryBuilder::insert().nodes().count(100).query().into()];
    server.api.db_exec_mut(owner, db, queries).await?;
    let (_, list) = server.api.db_list().await?;
    let original_size = list[0].size;
    let (status, db) = server.api.db_clear(owner, db, DbResource::Db).await?;
    assert_eq!(status, 200);
    assert!(db.size < original_size);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn clear_db_memory() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.user_login(owner).await?;
    server.api.db_add(owner, db, DbKind::Memory).await?;
    let queries = &[QueryBuilder::insert().nodes().count(100).query().into()];
    server.api.db_exec_mut(owner, db, queries).await?;
    let (_, list) = server.api.db_list().await?;
    let original_size = list[0].size;
    let (status, db) = server.api.db_clear(owner, db, DbResource::Db).await?;
    assert_eq!(status, 200);
    assert!(db.size < original_size);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn clear_db_memory_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.user_login(owner).await?;
    server.api.db_add(owner, db, DbKind::Memory).await?;
    let db_path = Path::new(&server.data_dir).join(owner).join(db);
    assert!(!db_path.exists());
    server.api.db_backup(owner, db).await?;
    assert!(db_path.exists());
    let (status, db) = server.api.db_clear(owner, db, DbResource::Backup).await?;
    assert!(!db_path.exists());
    assert_eq!(status, 200);
    assert_eq!(db.backup, 0);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn clear_all() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.user_login(owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let queries = &[QueryBuilder::insert().nodes().count(100).query().into()];
    server.api.db_exec_mut(owner, db, queries).await?;
    let (_, list) = server.api.db_list().await?;
    let original_size = list[0].size;
    server.api.db_backup(owner, db).await?;
    let (status, database) = server.api.db_clear(owner, db, DbResource::All).await?;
    assert_eq!(status, 200);
    assert!(database.size < original_size);
    let db_audit_file = Path::new(&server.data_dir)
        .join(owner)
        .join("audit")
        .join(format!("{db}.log"));
    assert!(!db_audit_file.exists());
    assert_eq!(database.backup, 0);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbKind::Memory).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    server.user_login(user).await?;
    let status = server
        .api
        .db_clear(owner, db, DbResource::All)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .db_clear("owner", "db", DbResource::All)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __clear_backup_type_def(),
        __clear_audit_type_def(),
        __clear_db_type_def(),
        __clear_db_memory_type_def(),
        __clear_db_memory_backup_type_def(),
        __clear_all_type_def(),
        __non_admin_type_def(),
        __no_token_type_def(),
    ]
}
