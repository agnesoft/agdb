use agdb::Comparison;
use agdb::DbError;
use agdb::DbMemory;
use agdb::DbType;
use agdb::DbTypeMarker;
use agdb::DbValue;
use agdb::QueryBuilder;

// Enums cannot be derived from agdb::DbType
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

// The empty trait `DbTypeMarker` (deriveable via
// `agdb::DbTypeMarker`) is used to allow the user
// types (structs & enums) to be used in vectorized
// variants of the user types.
#[derive(Debug, Clone, DbTypeMarker)]
struct Property {
    name: String,
    value: String,
}

// Deriving from agdb::DbType to make it possible
// to use directly in the database queries. Here we
// demonstrate usiage of custom values, vectorized
// custom values and optional values.
#[derive(Debug, DbType)]
struct User {
    username: String,
    password: String,
    display_name: Option<String>,
    status: UserStatus,
    extra: Vec<Property>,
}

// Example implementations of traits expected by agdb::DbType.
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

impl TryFrom<DbValue> for Property {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        let (name, value) = value.string()?.split_once("=").ok_or("Invalid property")?;

        Ok(Self {
            name: name.to_string(),
            value: value.to_string(),
        })
    }
}

impl From<Property> for DbValue {
    fn from(value: Property) -> Self {
        format!("{}={}", value.name, value.value).into()
    }
}

fn main() -> Result<(), DbError> {
    // Creates in memory database.
    let mut db = DbMemory::new("agdb_example")?;

    // Inserts root node for users with an alias. You can loosely
    // think of it akin to a table in relational databases.
    db.exec_mut(QueryBuilder::insert().nodes().aliases(["users"]).query())?;

    // Prepare some data to be inserted into the database. In this
    // case a couple of users.
    let users = vec![
        User {
            username: "user1".to_string(),
            password: "password123".to_string(),
            display_name: None,
            status: UserStatus::Active,
            extra: vec![Property {
                name: "email".to_string(),
                value: "user1@example.com".to_string(),
            }],
        },
        User {
            username: "user2".to_string(),
            password: "password456".to_string(),
            display_name: Some("DbUser2".into()),
            status: UserStatus::Inactive,
            extra: vec![],
        },
    ];

    // Inserts the users as new nodes into the database. This is made
    // possible by deriving `agdb::DbType` for the `User` struct.
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
    // Internally it uses the `agdb::DbType::db_keys()` to select required keys as the `User` is
    // required to implement the `DbType` trait. It also changes the sarch algorithm to depth-first
    // (default is breadth-first) which is more efficient when searching for a single item only. We could
    // additionally limit the search by adding `limit(1)` to ensure we get only one result back
    // and/or additional condition on `distance(Equal(2))` to stop search beyond users. But since
    // we know the structure of our data (graph) we can safely omit them as unnecessary here.
    //
    // Finally we convert the result into a `User` struct which is again possible due
    // to deriving agdb::DbType.
    let user: User = db
        .exec(
            QueryBuilder::select()
                .elements::<User>()
                .search()
                .depth_first()
                .from("users")
                .where_()
                .key("username")
                .value(Comparison::Equal("user1".into()))
                .query(),
        )?
        .try_into()?;

    println!("{user:?}");

    Ok(())
}
