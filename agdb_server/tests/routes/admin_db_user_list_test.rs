use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn list() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_user_list_test::list().await
}

#[tokio::test]
async fn db_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_user_list_test::db_not_found().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_user_list_test::non_admin().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_user_list_test::no_token().await
}
