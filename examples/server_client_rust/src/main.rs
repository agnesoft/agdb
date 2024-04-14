use agdb::Comparison;
use agdb::QueryBuilder;
use agdb::UserValue;
use agdb_api::DbType;
use agdb_api::ReqwestClient;

#[derive(Debug, UserValue)]
struct User {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Requires the server to be running. Run it with `cargo run -p agdb_server`
    // from the root.

    // Creates a client connecting to the remote server.
    let mut client = agdb_api::AgdbApi::new(ReqwestClient::new(), "localhost:3000");

    // Creates a user using default admin credentials.
    client.user_login("admin", "admin").await?;
    client.admin_user_add("client", "password111").await?;

    // Creates a database using the newly created user.
    client.user_login("client", "password111").await?; //overwrites the internal authorization token of the admin to client
    client.db_add("client", "db", DbType::Memory).await?;

    // Prepare some data to be inserted into the remote database
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

    // Prepare the queries to be executed on the remote database.
    let queries = vec![
        QueryBuilder::insert()
            .nodes()
            .aliases(vec!["users"])
            .query()
            .into(),
        QueryBuilder::insert().nodes().values(&users).query().into(),
    ];

    // Execute the first batch of queries.
    let results = client.db_exec("client", "db", &queries).await?.1;

    // Prepare the second batch using the result of the previous batch.
    let queries = vec![
        QueryBuilder::insert()
            .edges()
            .from("users")
            .to(results[1].ids())
            .query()
            .into(),
        QueryBuilder::select()
            .ids(
                QueryBuilder::search()
                    .depth_first()
                    .from("users")
                    .where_()
                    .key("username")
                    .value(Comparison::Equal("user1".into()))
                    .query(),
            )
            .query()
            .into(),
    ];

    // Execute the second batch of queries.
    let results = client.db_exec("client", "db", &queries).await?.1;

    // Print the result of the second query.
    println!("User: {:?}", results[1].elements[0].id);
    for key_value in results[1].elements[0].values.iter() {
        println!("  {}: {}", key_value.key, key_value.value);
    }

    Ok(())
}
