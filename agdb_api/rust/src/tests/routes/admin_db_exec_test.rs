use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryResult;
use crate::DbKind;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn read_write() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let queries = &[
        QueryBuilder::insert()
            .nodes()
            .aliases("root")
            .values([[("key", 1.1).into()]])
            .query()
            .into(),
        QueryBuilder::select().ids("root").query().into(),
    ];
    let (status, results) = server.api.admin_db_exec_mut(owner, db, queries).await?;
    assert_eq!(status, 200);
    let expected = vec![
        QueryResult {
            result: 1,
            elements: vec![DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![],
            }],
        },
        QueryResult {
            result: 1,
            elements: vec![DbElement {
                id: DbId(1),
                from: None,
                to: None,
                values: vec![("key", 1.1).into()],
            }],
        },
    ];
    assert_eq!(results, expected);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn read_only() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let queries = &[QueryBuilder::insert()
        .nodes()
        .aliases("root")
        .values([[("key", 1.1).into()]])
        .query()
        .into()];
    let (status, _) = server.api.admin_db_exec_mut(owner, db, queries).await?;
    assert_eq!(status, 200);
    let queries = &[QueryBuilder::select().ids("root").query().into()];
    let (status, results) = server.api.admin_db_exec(owner, db, queries).await?;
    assert_eq!(status, 200);
    let expected = vec![QueryResult {
        result: 1,
        elements: vec![DbElement {
            id: DbId(1),
            from: None,
            to: None,
            values: vec![("key", 1.1).into()],
        }],
    }];
    assert_eq!(results, expected);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn query_error() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let queries = &[
        QueryBuilder::insert()
            .nodes()
            .values([[("key", 1.1).into()]])
            .query()
            .into(),
        QueryBuilder::select().ids("root").query().into(),
    ];
    let error = server
        .api
        .admin_db_exec_mut(owner, db, queries)
        .await
        .unwrap_err();
    assert_eq!(error.status, 470);
    assert_eq!(error.description, "Alias 'root' not found");
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn db_not_found() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_db_exec("owner", "db", &[])
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
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
        .admin_db_exec(owner, "db", &[])
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_db_exec("owner", "db", &[])
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __read_write_type_def(),
        __read_only_type_def(),
        __query_error_type_def(),
        __db_not_found_type_def(),
        __non_admin_type_def(),
        __no_token_type_def(),
    ]
}
