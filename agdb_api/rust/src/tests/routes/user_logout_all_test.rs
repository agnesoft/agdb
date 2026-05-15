use crate::AgdbApi;
use crate::ReqwestClient;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_user_name;
use crate::test_server::reqwest_client;
use crate::test_server::test_error::TestError;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn logout_all() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();

    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;

    let mut client1 = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        server.api.address(),
    );
    let mut client2 = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        server.api.address(),
    );

    client1.user_login(user, user).await?;
    client2.user_login(user, user).await?;

    client1.user_status().await?;
    client2.user_status().await?;

    let status = client1.user_logout_all().await?;
    assert_eq!(status, 201);
    assert_eq!(client1.token, None);

    assert_eq!(client1.user_status().await.unwrap_err().status, 401);
    assert_eq!(client2.user_status().await.unwrap_err().status, 401);

    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn no_token() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let status = server.api.user_logout_all().await.unwrap_err().status;
    assert_eq!(status, 401);

    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![__logout_all_type_def(), __no_token_type_def()]
}
