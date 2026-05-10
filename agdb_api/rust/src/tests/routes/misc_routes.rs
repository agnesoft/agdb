use crate::test_server::TestServer;
use crate::test_server::reqwest_client;
use crate::test_server::test_error::TestError;
use reqwest::StatusCode;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn missing() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let client = reqwest_client();
    let status = client
        .get(server.full_url("/missing"))
        .send()
        .await?
        .status();
    assert_eq!(status, StatusCode::NOT_FOUND);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn status() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server.api.status().await?;
    assert_eq!(status, 200);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn openapi() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let client = reqwest_client();
    let status = client
        .get(server.full_url("/openapi.json"))
        .send()
        .await?
        .status();
    assert_eq!(status, 200);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __missing_type_def(),
        __status_type_def(),
        __openapi_type_def(),
    ]
}
