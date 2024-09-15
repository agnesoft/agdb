use agdb::Comparison;
use agdb::DbError;
use agdb::DbMemory;
use agdb::DbUserValue;
use agdb::DbValue;
use agdb::QueryBuilder;
use agdb::QueryError;
use agdb::UserValue;

// Enums cannot be derived from agdb::UserValue
// directly because the underlying data type is
// not known and can differ from case to case.
// Even for the simplest of enums like this one
// it can be stored in multiple ways. As strings
// like in this example, as integers integers or
// possibly other formats (e.g. bytes). Derived
// implementation would have to make that call which
// is better to be left on the user.
#[derive(Debug, Clone)]
enum UserStatus {
    Active,
    Inactive,
    Banned,
}

// Deriving from agdb::UserValue to make it possible
// to use directly in the database queries.
#[derive(Debug, UserValue)]
struct User {
    username: String,
    password: String,
    status: UserStatus,
}

// Example implementation of traits expected by agdb::UserValue.
impl TryFrom<DbValue> for UserStatus {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        match value.to_string().as_str() {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            "banned" => Ok(Self::Banned),
            _ => Err(DbError::from("Invalid user status")),
        }
    }
}
impl From<UserStatus> for DbValue {
    fn from(value: UserStatus) -> Self {
        match value {
            UserStatus::Active => "active".into(),
            UserStatus::Inactive => "inactive".into(),
            UserStatus::Banned => "banned".into(),
        }
    }
}

fn main() -> Result<(), QueryError> {
    // Creates in memory database.
    let mut db = DbMemory::new("agdb_example")?;

    // Inserts root node for users with an alias. You can loosely
    // think of it akin to a table in relational databases.
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .aliases(vec!["users"])
            .query(),
    )?;

    // Prepare some data to be inserted into the database. In this
    // case a couple of users.
    let users = vec![
        User {
            username: "user1".to_string(),
            password: "password123".to_string(),
            status: UserStatus::Active,
        },
        User {
            username: "user2".to_string(),
            password: "password456".to_string(),
            status: UserStatus::Inactive,
        },
    ];

    // Inserts the users as new nodes into the database. This is made
    // possible by deriving `agdb::UserValue` for the `User` struct.
    let users = db.exec_mut(QueryBuilder::insert().nodes().values(&users).query())?;

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
    // SELECT username, password, status FROM users WHERE name = "user1"
    // ```
    //
    // It uses the `agdb::UserValue::db_keys()` to select required keys. It also
    // changes the sarch algorithm to depth-first (default is breadth-first)
    // which is more efficient when searching for a single item only. We could additionally
    // limit the search by adding `limit(1)` to ensure we get only one result back
    // and/or additional condition on `distance(Equal(2))` to stop search beyond users. But since
    // we know the structure of our data (graph) we can safely omit them as unnecessary here.
    //
    // Finally we convert the result into a `User` struct which is again possible due
    // to deriving agdb::UserValue.
    let user: User = db
        .exec(
            QueryBuilder::select()
                .values(User::db_keys())
                .ids(
                    QueryBuilder::search()
                        .depth_first()
                        .from("users")
                        .where_()
                        .key("username")
                        .value(Comparison::Equal("user1".into()))
                        .query(),
                )
                .query(),
        )?
        .try_into()?;

    println!("{:?}", user);

    Ok(())
}
