use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn read_write() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_exec_test::read_write().await
}

#[tokio::test]
async fn read_only() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_exec_test::read_only().await
}

#[tokio::test]
async fn query_error() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_exec_test::query_error().await
}

#[tokio::test]
async fn db_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_exec_test::db_not_found().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_exec_test::non_admin().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_exec_test::no_token().await
}

