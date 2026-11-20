use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb_api::DbKind;
use agdb_api::DbResource;
use std::path::Path;

async fn clear_db_memory_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Memory).await?;
    let db_path = Path::new(&server.data_dir).join(owner).join(db);
    let backup_path = Path::new(&server.data_dir)
        .join(owner)
        .join("backups")
        .join(format!("{db}.bak"));
    assert!(!db_path.exists());
    server.api.db_backup(owner, db).await?;
    assert!(backup_path.exists());
    let (status, db) = server.api.db_clear(owner, db, DbResource::Backup).await?;
    assert!(!backup_path.exists());
    assert_eq!(status, 200);
    assert_eq!(db.backup, 0);
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![clear_db_memory_backup,];
