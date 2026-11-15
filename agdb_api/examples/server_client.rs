//! Example: connect to an agdb server with `agdb_api`, authenticate, create a
//! database, and run a batch of typed queries against it.

use agdb::DbType;
use agdb::QueryBuilder;
use agdb_api::DbKind as ApiDbType;
use agdb_api::ReqwestClient;

#[derive(Debug, DbType)]
struct User {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Requires the server to be running. Run it with `cargo run -p agdb_server`
    // from the root.

    let mut client = agdb_api::AgdbApi::new(ReqwestClient::new(), "localhost:3000");

    // Log in as admin and create a new user.
    client.user_login("admin", "admin").await?;
    client.admin_user_add("client", "password111").await?;

    // Switch to the new user and create a database.
    client.user_login("client", "password111").await?;
    client.db_add("client", "db", ApiDbType::Memory).await?;

    let users = vec![
        User {
            username: "user1".to_string(),
            password: "password123".to_string(),
        },
        User {
            username: "user2".to_string(),
            password: "password456".to_string(),
        },
    ];

    // Insert a root "users" alias and the user nodes.
    let queries = vec![
        QueryBuilder::insert()
            .nodes()
            .aliases(["users"])
            .query()
            .into(),
        QueryBuilder::insert().nodes().values(&users).query().into(),
    ];
    let results = client.db_exec_mut("client", "db", &queries).await?.1;

    // Link users to root and search for one by username.
    let queries = vec![
        QueryBuilder::insert()
            .edges()
            .from("users")
            .to(results[1].ids())
            .query()
            .into(),
        QueryBuilder::select()
            .search()
            .depth_first()
            .from("users")
            .where_()
            .key("username")
            .value("user1")
            .query()
            .into(),
    ];

    let results = client.db_exec_mut("client", "db", &queries).await?.1;

    println!("User: {:?}", results[1].elements[0].id);
    for key_value in results[1].elements[0].values.iter() {
        println!("  {}: {}", key_value.key, key_value.value);
    }

    Ok(())
}
