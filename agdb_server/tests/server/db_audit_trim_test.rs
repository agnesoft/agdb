use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::audit_dir;
use crate::test_server::audit_entries;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use crate::test_server::write_audit_entry;
use agdb::QueryBuilder;
use agdb_api::DbKind;

const DAY_2020_01_01: u64 = 1_577_836_800;
const KEEP_DAYS: u64 = 1;

async fn audit_trim() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let queries = &[QueryBuilder::insert().nodes().count(1).query().into()];
    server.api.db_exec_mut(owner, db, queries).await?;
    let audit = audit_dir(&server.data_dir, owner, db);
    write_audit_entry(&audit, "2020-01-01", DAY_2020_01_01)?;

    let status = server.api.db_audit_trim(owner, db, KEEP_DAYS).await?;

    assert_eq!(status, 200);
    assert_eq!(audit_entries(&audit)?, 1);
    assert_eq!(server.api.db_audit(owner, db).await?.1.0.len(), 1);
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![audit_trim,];
