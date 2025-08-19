use agdb::CountComparison;
use agdb::DbError;
use agdb::DbId;
use agdb::DbMemory;
use agdb::DbType;
use agdb::DbValue;
use agdb::QueryBuilder;

#[derive(Debug, DbType)]
struct UserDb {
    name: String,
    status: String,
    age: u64,
}

#[derive(Debug, Clone)]
enum UserStatus {
    Active,
    Inactive,
    Banned,
}

#[derive(Debug, DbType)]
struct UserDb2 {
    db_id: Option<DbId>,
    name: String,
    status: UserStatus,
}

impl TryFrom<DbValue> for UserStatus {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        match value.to_string().as_str() {
            "active" | "0" => Ok(Self::Active),
            "inactive" | "1" => Ok(Self::Inactive),
            "banned" | "2" => Ok(Self::Banned),
            _ => Err(DbError::from("Invalid user status")),
        }
    }
}
impl From<UserStatus> for DbValue {
    fn from(value: UserStatus) -> Self {
        match value {
            UserStatus::Active => 0_u64.into(),
            UserStatus::Inactive => 1_u64.into(),
            UserStatus::Banned => 2_u64.into(),
        }
    }
}

fn main() -> Result<(), DbError> {
    // Creates in memory database.
    let mut db = DbMemory::new("agdb_example")?;

    // Inserts root nodes for users.
    db.exec_mut(QueryBuilder::insert().nodes().aliases(["users"]).query())?;

    let mut users = vec![];

    // Prepare users using schema of UserDb.
    for i in 0..100 {
        let status = if i % 3 == 0 {
            "active"
        } else if i % 3 == 1 {
            "inactive"
        } else {
            "banned"
        }
        .to_string();

        users.push(UserDb {
            name: format!("user{i}"),
            status,
            age: i,
        });
    }

    let users = db.exec_mut(QueryBuilder::insert().nodes().values(&users).query())?;

    // Attach the users to the users node.
    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from("users")
            .to(users)
            .query(),
    )?;

    let user = db.exec(QueryBuilder::select().ids(50).query())?;
    println!("{user:?}");

    // Migrating the schema from UserDb to UserDb2. The difference is that "age" property
    // is no longer there and the type of the "status" property changed from String to
    // the UserStatus enum. We will do it in few steps all in a transaction.
    db.transaction_mut(|t| {
        // First remove the "age" property from all the users.
        t.exec_mut(
            QueryBuilder::remove()
                .values("age")
                .ids(QueryBuilder::search().from("users").query())
                .query(),
        )?;

        // Next we can use a small trick. In the implementation of the `TryFrom<DbValue>` for
        // the `UserStatus` enum we have defined that the string literal values present in the
        // database match to the corresponding values of the enum. We can therefore load all the
        // users using the new schema.
        let users: Vec<UserDb2> = t
            .exec(
                QueryBuilder::select()
                    .search()
                    .from("users")
                    .where_()
                    .distance(CountComparison::Equal(2))
                    .query(),
            )?
            .try_into()?;

        // And finally since we have defined the `From<UserStatus> for DbValue` to represent the
        // enum as numbers, we can simply re-instert the users back.
        t.exec_mut(QueryBuilder::insert().elements(&users).query())

        // NOTE: When migrating huge amount of data it would be better to do it in batches using
        // the `limit` and `offset` values in the search query.
    })?;

    let user = db.exec(QueryBuilder::select().ids(50).query())?;
    println!("{user:?}");

    // Finally after you are sure the data in the "old" format is no longer needed you can remove
    // it from the code in any of the subsequent releases of your software.

    Ok(())
}
