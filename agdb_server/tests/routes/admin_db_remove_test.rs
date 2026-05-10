use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn remove() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_remove_test::remove().await
}

#[tokio::test]
async fn db_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_remove_test::db_not_found().await
}

#[tokio::test]
async fn user_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_remove_test::user_not_found().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_remove_test::non_admin().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_remove_test::no_token().await
}

