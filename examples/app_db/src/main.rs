use agdb::Comparison;
use agdb::DbError;
use agdb::DbMemory;
use agdb::QueryBuilder;

fn main() -> Result<(), DbError> {
    // Creates in memory database.
    let mut db = DbMemory::new("agdb_example")?;

    // Inserts root node for users with an alias. You can loosely
    // think of it akin to a table in relational databases.
    db.exec_mut(QueryBuilder::insert().nodes().aliases(["users"]).query())?;

    // Inserts the raw data by providing list of keys and values
    // for each node. Notice we can easily create sparse data
    // omitting certain fields as there is no schema to enforce.
    let users = db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([
                vec![
                    ("username", "user1").into(),
                    ("password", "password123").into(),
                    ("age", 20).into(),
                ],
                vec![
                    ("username", "user2").into(),
                    ("password", "password456").into(),
                ],
            ])
            .query(),
    )?;

    // Link the users with the "users" node. Note that we are referring to
    // the users node with an alias "users" we have given it and to
    // the new user nodes by supplying the result of the previous query directly.
    // One could also use the `QueryResult::ids()` method to extract the ids.
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from("users")
            .to(users)
            .query(),
    )?;

    // Query the database searching for a particular user. Note that this query
    // is nested. It selects from a result of a search query. The nested queries
    // like this are atomic - part of the same underlying transaction. You can also
    // make the transaction explicit with `DbImpl::transaction()` instead of DbImpl::exec()
    // if you need chaining queries or additional processing around them.
    //
    // This query would be roughly equivalent to the following SQL:
    //
    // ```
    // SELECT * FROM users WHERE name = "user1"
    // ```
    //
    // It changes the search algorithm to depth-first (default is breadth-first)
    // which is more efficient when searching for a single item only. We could additionally
    // limit the search by adding `limit(1)` to ensure we get only one result back
    // and/or additional condition on `distance(Equal(2))` to stop search beyond users. But since
    // we know the structure of our data (graph) we can safely omit them as unnecessary here.
    let user_result = db.exec(
        QueryBuilder::select()
            .search()
            .depth_first()
            .from("users")
            .where_()
            .key("username")
            .value(Comparison::Equal("user1".into()))
            .query(),
    )?;

    // Following demonstrates an idea how to handle raw database
    // result. It has always the same structure. The object contains
    // numerical result in `QueryResult::result` and list of elements
    // in `QueryResult::elements`. Each element has an id and list of
    // properties (key-value pairs). You can differentiate nodes and
    // edges by checking the id. Nodes have positive ids, edges have
    // negative ids.
    if user_result.result == 1 {
        println!("User: {:?}", user_result.elements[0].id);

        for key_value in user_result.elements[0].values.iter() {
            println!("  {}: {}", key_value.key, key_value.value);
        }
    }

    Ok(())
}
