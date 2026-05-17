use crate::DbKind;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb::QueryBuilder;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn admin_audit() -> Result<(), TestError> {
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
    server.api.user_login(ADMIN, ADMIN).await?;
    let (status, results) = server.api.admin_db_audit(owner, db).await?;
    assert_eq!(status, 200);
    assert_eq!(results.0[0].username, owner.to_string());
    assert_eq!(results.0[0].query, queries.remove(0));
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn admin_audit_db_empty() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let (status, results) = server.api.admin_db_audit(owner, db).await?;
    assert_eq!(status, 200);
    assert_eq!(results.0, vec![]);
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
        .admin_db_audit("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn audit_no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_db_audit("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __admin_audit_type_def(),
        __admin_audit_db_empty_type_def(),
        __non_admin_type_def(),
        __audit_no_token_type_def(),
    ]
}
