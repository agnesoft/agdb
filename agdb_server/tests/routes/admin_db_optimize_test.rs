use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn optimize() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_optimize_test::optimize().await
}

#[tokio::test]
async fn shrink_to_fit() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_optimize_test::shrink_to_fit().await
}

#[tokio::test]
async fn db_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_optimize_test::db_not_found().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_optimize_test::non_admin().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_optimize_test::no_token().await
}

