use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn add_get_list() -> Result<(), TestError> {
    agdb_api::tests::routes::dashboard_test::add_get_list().await
}

#[tokio::test]
async fn add_existing() -> Result<(), TestError> {
    agdb_api::tests::routes::dashboard_test::add_existing().await
}

#[tokio::test]
async fn add_invalid_name() -> Result<(), TestError> {
    agdb_api::tests::routes::dashboard_test::add_invalid_name().await
}

#[tokio::test]
async fn save_upserts() -> Result<(), TestError> {
    agdb_api::tests::routes::dashboard_test::save_upserts().await
}

#[tokio::test]
async fn rename() -> Result<(), TestError> {
    agdb_api::tests::routes::dashboard_test::rename().await
}

#[tokio::test]
async fn remove() -> Result<(), TestError> {
    agdb_api::tests::routes::dashboard_test::remove().await
}

#[tokio::test]
async fn other_user() -> Result<(), TestError> {
    agdb_api::tests::routes::dashboard_test::other_user().await
}

#[tokio::test]
async fn removed_with_user() -> Result<(), TestError> {
    agdb_api::tests::routes::dashboard_test::removed_with_user().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::dashboard_test::no_token().await
}
