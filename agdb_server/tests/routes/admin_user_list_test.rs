use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn user_list() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_list_test::user_list().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_list_test::non_admin().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_list_test::no_token().await
}
