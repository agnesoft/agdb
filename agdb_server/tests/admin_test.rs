pub mod framework;

use crate::framework::TestServer;
use crate::framework::NO_TOKEN;
use std::collections::HashMap;

#[tokio::test]
async fn create_user() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let mut user = HashMap::new();
    user.insert("name", "a");
    user.insert("password", "");
    assert_eq!(
        server.post("/admin/create_user", &user, NO_TOKEN).await?.0,
        401
    ); //permission denied
    let bad_token = Some("bad".to_string());
    assert_eq!(
        server
            .post("/admin/create_user", &user, &bad_token)
            .await?
            .0,
        403
    ); //forbidden
    let admin_token = server.init_admin().await?;
    assert_eq!(
        server
            .post("/admin/create_user", &user, &admin_token)
            .await?
            .0,
        461
    ); //short name
    user.insert("name", "alice");
    assert_eq!(
        server
            .post("/admin/create_user", &user, &admin_token)
            .await?
            .0,
        462
    ); //short password
    user.insert("password", "mypassword123");
    assert_eq!(
        server
            .post("/admin/create_user", &user, &admin_token)
            .await?
            .0,
        201
    ); //created
    assert_eq!(
        server
            .post("/admin/create_user", &user, &admin_token)
            .await?
            .0,
        463
    ); //user exists
    Ok(())
}
