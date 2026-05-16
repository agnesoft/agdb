use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn logout() -> Result<(), TestError> {
    agdb_api::tests::routes::user_logout_test::logout().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::user_logout_test::no_token().await
}

#[tokio::test]
async fn logout_only_current_user_token() -> Result<(), TestError> {
    agdb_api::tests::routes::user_logout_test::logout_only_current_user_token().await
}

#[tokio::test]
async fn logout_selected_session() -> Result<(), TestError> {
    agdb_api::tests::routes::user_logout_test::logout_selected_session().await
}
