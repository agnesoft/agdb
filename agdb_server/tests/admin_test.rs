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

#[tokio::test]
async fn change_password() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let admin_token = server.init_admin().await?;

    let mut user = HashMap::new();
    user.insert("name", "alice");
    user.insert("password", "password123");

    assert_eq!(
        server
            .post("/admin/create_user", &user, &admin_token)
            .await?
            .0,
        201
    );

    user.insert("password", "password456");

    let bad_token = Some("bad".to_string());
    assert_eq!(
        server
            .post("/admin/change_password", &user, &bad_token)
            .await?
            .0,
        403
    );

    assert_eq!(
        server
            .post("/admin/change_password", &user, &admin_token)
            .await?
            .0,
        200
    );

    assert_eq!(server.post("/user/login", &user, NO_TOKEN).await?.0, 200);

    user.insert("name", "bob");

    assert_eq!(
        server
            .post("/admin/change_password", &user, &admin_token)
            .await?
            .0,
        403
    );

    user.insert("password", "pass");

    assert_eq!(
        server
            .post("/admin/change_password", &user, &admin_token)
            .await?
            .0,
        462
    );

    Ok(())
}
