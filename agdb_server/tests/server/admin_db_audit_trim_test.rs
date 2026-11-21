use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::audit_dir;
use crate::test_server::audit_entries;
use crate::test_server::backup_audit_dir;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use crate::test_server::write_audit_entry;
use agdb::QueryBuilder;
use agdb_api::DbKind;
use std::path::Path;

const DAY_2020_01_01: u64 = 1_577_836_800;
const KEEP_DAYS: u64 = 1;

async fn audit_trim() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let queries = &[QueryBuilder::insert().nodes().count(1).query().into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    server.api.admin_db_backup(owner, db).await?;
    let audit = audit_dir(&server.data_dir, owner, db);
    let backup_audit = backup_audit_dir(&server.data_dir, owner, db);
    write_audit_entry(&audit, "2020-01-01", DAY_2020_01_01)?;
    write_audit_entry(&backup_audit, "2020-01-01", DAY_2020_01_01)?;
    assert_eq!(server.api.admin_db_audit(owner, db).await?.1.0.len(), 2);

    let status = server.api.admin_db_audit_trim(owner, db, KEEP_DAYS).await?;

    assert_eq!(status, 200);
    assert_eq!(audit_entries(&audit)?, 1);
    assert_eq!(audit_entries(&backup_audit)?, 1);
    assert!(!Path::new(&audit).join("2020-01-01.log").exists());
    let (_, entries) = server.api.admin_db_audit(owner, db).await?;
    assert_eq!(entries.0.len(), 1);
    assert_eq!(entries.0[0].query, queries[0]);
    Ok(())
}

async fn audit_trim_keep_all() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Memory).await?;
    let queries = &[QueryBuilder::insert().nodes().count(1).query().into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let audit = audit_dir(&server.data_dir, owner, db);
    write_audit_entry(&audit, "2020-01-01", DAY_2020_01_01)?;

    let status = server.api.admin_db_audit_trim(owner, db, u64::MAX).await?;

    assert_eq!(status, 200);
    assert_eq!(server.api.admin_db_audit(owner, db).await?.1.0.len(), 2);
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![audit_trim, audit_trim_keep_all,];
