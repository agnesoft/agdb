use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn user_list() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user1 = &next_user_name();
    let user2 = &next_user_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(user1, user1).await?;
    server.api.admin_user_add(user2, user2).await?;
    let (status, mut list) = server.api.admin_user_list().await?;
    list.sort();
    assert_eq!(status, 200);
    let admin_user = &list[0];
    assert_eq!(admin_user.username, "admin");
    assert!(admin_user.login);
    assert!(admin_user.admin);
    assert!(!admin_user.sessions.is_empty());
    assert_eq!(admin_user.sessions[0].agent, "agdb_api");
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.user_login(user).await?;
    let status = server.api.admin_user_list().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server.api.admin_user_list().await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __user_list_type_def(),
        __non_admin_type_def(),
        __no_token_type_def(),
    ]
}
