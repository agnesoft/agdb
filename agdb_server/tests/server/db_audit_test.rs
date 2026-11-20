use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb::QueryBuilder;
use agdb_api::DbKind;
use std::path::Path;

async fn audit_delete_db() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let queries = vec![
        QueryBuilder::insert()
            .nodes()
            .aliases("root")
            .values([[("key", 1.1).into()]])
            .query()
            .into(),
    ];
    server.api.db_exec_mut(owner, db, &queries).await?;
    let db_audit_dir = Path::new(&server.data_dir)
        .join(owner)
        .join("audit")
        .join(db);
    assert!(db_audit_dir.exists());
    let status = server.api.db_delete(owner, db).await?;
    assert_eq!(status, 204);
    assert!(!db_audit_dir.exists());
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![audit_delete_db,];
