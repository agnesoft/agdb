use crate::test_db::TestFile;
use agdb::QueryError;
use agdb::{Comparison::Equal, Db, DbId, QueryBuilder, UserValue};

#[test]
fn quickstart() -> Result<(), QueryError> {
    let _test_file = TestFile::from("db_file.agdb");
    let mut db = Db::new("db_file.agdb")?;

    db.exec_mut(QueryBuilder::insert().nodes().aliases("users").query())?;

    #[derive(Debug, UserValue)]
    struct User {
        db_id: Option<DbId>,
        name: String,
    }
    let users = vec![
        User {
            db_id: None,
            name: "Alice".into(),
        },
        User {
            db_id: None,
            name: "Bob".into(),
        },
        User {
            db_id: None,
            name: "John".into(),
        },
    ];

    let users_ids = db.exec_mut(QueryBuilder::insert().nodes().values(&users).query())?;

    db.exec_mut(
        QueryBuilder::insert()
            .edges()
            .from("users")
            .to(&users_ids)
            .query(),
    )?;

    let users: Vec<User> = db
        .exec(
            QueryBuilder::select()
                .elements::<User>()
                .ids(&users_ids)
                .query(),
        )?
        .try_into()?;

    println!("{users:?}");
    // [User { db_id: Some(DbId(2)), username: "Alice" },
    //  User { db_id: Some(DbId(3)), username: "Bob" },
    //  User { db_id: Some(DbId(4)), username: "John" }]

    let user: User = db
        .exec(
            QueryBuilder::select()
                .elements::<User>()
                .search()
                .from("users")
                .where_()
                .key("name")
                .value(Equal("Bob".into()))
                .query(),
        )?
        .try_into()?;

    println!("{user:?}");
    // User { db_id: Some(DbId(3)), username: "Bob" }

    Ok(())
}
