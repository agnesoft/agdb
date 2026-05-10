use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn add_db_user() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_add_test::add_db_user().await
}

#[tokio::test]
async fn add_db_user_as_non_owner_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_add_test::add_db_user_as_non_owner_admin().await
}

#[tokio::test]
async fn change_user_role() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_add_test::change_user_role().await
}

#[tokio::test]
async fn db_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_add_test::db_not_found().await
}

#[tokio::test]
async fn user_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_add_test::user_not_found().await
}

#[tokio::test]
async fn change_owner_role() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_add_test::change_owner_role().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_add_test::non_admin().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_add_test::no_token().await
}

