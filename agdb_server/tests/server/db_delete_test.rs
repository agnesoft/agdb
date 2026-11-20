use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb_api::DbKind;
use std::path::Path;

async fn delete() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    assert!(Path::new(&server.data_dir).join(owner).join(db).exists());
    let status = server.api.db_delete(owner, db).await?;
    assert_eq!(status, 204);
    assert!(!Path::new(&server.data_dir).join(owner).join(db).exists());
    Ok(())
}

async fn delete_with_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db_path = Path::new(&server.data_dir).join(owner).join(db);
    let db_backup_path = Path::new(&server.data_dir)
        .join(owner)
        .join("backups")
        .join(format!("{db}.bak"));
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
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

async fn delete_in_memory_with_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db_path = Path::new(&server.data_dir).join(owner).join(db);
    let db_backup_path = Path::new(&server.data_dir)
        .join(owner)
        .join("backups")
        .join(format!("{db}.bak"));
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Memory).await?;
    assert!(!db_path.exists());
    server.api.db_backup(owner, db).await?;
    assert!(db_backup_path.exists());
    let status = server.api.db_delete(owner, db).await?;
    assert!(!db_backup_path.exists());
    assert_eq!(status, 204);
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![delete, delete_with_backup, delete_in_memory_with_backup,];
