use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServerImpl;
use crate::test_server::next_user_name;
use crate::test_server::reqwest_client;
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;

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

pub(super) const TESTS: &[Test] = tests![logout_all,];
