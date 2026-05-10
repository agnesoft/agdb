use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn change_password() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_change_password_test::change_password().await
}

#[tokio::test]
async fn password_short() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_change_password_test::password_short().await
}

#[tokio::test]
async fn user_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_change_password_test::user_not_found().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_change_password_test::non_admin().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_user_change_password_test::no_token().await
}

