use agdb::QueryBuilder;
use crate::DbKind;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use std::path::Path;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn audit() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let mut queries = vec![
        QueryBuilder::insert()
            .nodes()
            .aliases("root")
            .values([[("key", 1.1).into()]])
            .query()
            .into(),
        QueryBuilder::select().ids(":0").query().into(),
    ];
    server.api.db_exec_mut(owner, db, &queries).await?;
    let (status, results) = server.api.db_audit(owner, db).await?;
    assert_eq!(status, 200);
    assert_eq!(results.0[0].username, owner.to_string());
    assert_eq!(results.0[0].query, queries.remove(0));
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn audit_delete_db() -> Result<(), TestError> {
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
    let db_audit_file = Path::new(&server.data_dir)
        .join(owner)
        .join("audit")
        .join(format!("{db}.log"));
    assert!(db_audit_file.exists());
    let status = server.api.db_delete(owner, db).await?;
    assert_eq!(status, 204);
    assert!(!db_audit_file.exists());
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn audit_db_empty() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let (status, results) = server.api.db_audit(owner, db).await?;
    assert_eq!(status, 200);
    assert_eq!(results.0, vec![]);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn audit_no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server.api.db_audit("owner", "db").await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn repeated_query_with_db_audit() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    server
        .api
        .db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert()
                .nodes()
                .aliases("root")
                .query()
                .into()],
        )
        .await?;
    let (status, audit) = server.api.db_audit(owner, db).await?;
    assert_eq!(status, 200);
    assert_eq!(audit.0.len(), 1);
    server
        .api
        .db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert()
                .nodes()
                .aliases("root")
                .query()
                .into()],
        )
        .await?;
    let (status, audit2) = server.api.db_audit(owner, db).await?;
    assert_eq!(status, 200);
    assert_eq!(audit2.0.len(), 2);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __audit_type_def(),
        __audit_delete_db_type_def(),
        __audit_db_empty_type_def(),
        __audit_no_token_type_def(),
        __repeated_query_with_db_audit_type_def(),
    ]
}
