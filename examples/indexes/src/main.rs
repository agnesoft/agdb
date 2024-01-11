use agdb::DbMemory;
use agdb::DbUserValue;
use agdb::QueryBuilder;
use agdb::QueryError;
use agdb::UserValue;

// Deriving from agdb::UserValue to make it possible
// to use directly in the database queries.
#[derive(Debug, UserValue)]
struct User {
    username: String,
    password: String,
    token: String,
}

fn main() -> Result<(), QueryError> {
    // Creates in memory database.
    let mut db = DbMemory::new("agdb_example")?;

    // Create two indexes, one for username and one for token.
    db.exec_mut(&QueryBuilder::insert().index("username").query())?;
    db.exec_mut(&QueryBuilder::insert().index("token").query())?;

    // Inserts root node for users with an alias. You can loosely
    // think of it akin to a table in relational databases.
    db.exec_mut(
        &QueryBuilder::insert()
            .nodes()
            .aliases(vec!["users"])
            .query(),
    )?;

    // Create many users tied to the users node with keys "username" and "token"
    // so they get registered into the indexes. Note that indexes can be created
    // ex post as well and all existing elements with the relevant value(s) wiil
    // be automatically indexed.
    let mut users = vec![];
    for i in 0..100 {
        users.push(User {
            username: format!("user{}", i),
            password: format!("password{}", i),
            token: format!("token{}", i),
        });
    }
    let users = db.exec_mut(&QueryBuilder::insert().nodes().values(&users).query())?;
    db.exec_mut(
        &QueryBuilder::insert()
            .edges()
            .from("users")
            .to(users)
            .query(),
    )?;

    // Querying the database looking up a user with username "user50". Instead
    // of searching the graph we simply lookup the username in the index.
    //
    // Looking up index instead of graph search is useful when the data is too
    // uniform and cannot be split or easily modelled on the graph. For example
    // users that you want to lookup based on their username or logged in users
    // to be looked up using a token.
    let user: User = db
        .exec(
            &QueryBuilder::select()
                .values(User::db_keys())
                .ids(
                    QueryBuilder::search()
                        .index("username")
                        .value("user50")
                        .query(),
                )
                .query(),
        )?
        .try_into()?;

    println!("{:?}", user);

    Ok(())
}
