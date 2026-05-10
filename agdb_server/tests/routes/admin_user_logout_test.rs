use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn logout() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_logout_test::logout().await
}

#[tokio::test]
async fn unknown_user() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_logout_test::unknown_user().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_logout_test::non_admin().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_logout_test::no_token().await
}
