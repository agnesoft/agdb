use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn list() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_list_test::list().await
}

#[tokio::test]
async fn non_db_user() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_list_test::non_db_user().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_list_test::no_token().await
}
