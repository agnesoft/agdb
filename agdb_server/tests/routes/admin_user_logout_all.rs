use crate::next_user_name;
use crate::reqwest_client;
use crate::TestServer;
use crate::TestServerImpl;
use crate::ADMIN;
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;

#[tokio::test]
async fn logout_all() -> anyhow::Result<()> {
    let server = TestServerImpl::new().await?;
    let mut client = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &server.address,
    );
    let user = &next_user_name();
    let user2 = &next_user_name();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(user, user).await?;
    client.admin_user_add(user2, user2).await?;
    client.user_login(user, user).await?;
    client.user_login(user2, user2).await?;
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_logout_all().await?;

    let list = client.admin_user_list().await?.1;
    assert_eq!(list.iter().filter(|u| !u.admin && u.login).count(), 0);

    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server.api.admin_user_logout_all().await.unwrap_err().status;
    assert_eq!(status, 401);

    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server.api.admin_user_logout_all().await.unwrap_err().status;
    assert_eq!(status, 401);

    Ok(())
}
