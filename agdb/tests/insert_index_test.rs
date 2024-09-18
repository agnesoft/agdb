mod test_db;

use agdb::QueryBuilder;
use agdb::QueryError;
use test_db::TestDb;

#[test]
fn insert_empty_index() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("username").query(), 0);
}

#[test]
fn insert_index_with_existing_data() {
    let mut db = TestDb::new();
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([
                vec![("username", "user1").into(), ("age", 20).into()],
                vec![("username", "user2").into()],
                vec![("username", "user3").into(), ("age", 33).into()],
            ])
            .query(),
        3,
    );

    db.exec_mut(QueryBuilder::insert().index("username").query(), 3);
}

#[test]
fn insert_existing_index() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("username").query(), 0);
    db.exec_mut_error(
        QueryBuilder::insert().index("username").query(),
        "Index 'username' already exists",
    );
}

#[test]
fn insert_index_rollback() {
    let mut db = TestDb::new();
    db.transaction_mut_error(
        |t| {
            t.exec_mut(QueryBuilder::insert().index("username").query())?;
            t.exec_mut(
                QueryBuilder::insert()
                    .nodes()
                    .values([
                        [("username", "user1").into()],
                        [("username", "user2").into()],
                        [("username", "user3").into()],
                    ])
                    .query(),
            )?;
            t.exec_mut(QueryBuilder::insert().index("username").query())?;
            Ok(())
        },
        QueryError::from("Index 'username' already exists"),
    );
    db.exec(QueryBuilder::select().indexes().query(), 0);
}

#[test]
fn insert_indexed_value() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("username").query(), 0);
    db.exec_mut(QueryBuilder::insert().nodes().count(1).query(), 1);
    db.exec_mut(
        QueryBuilder::insert()
            .values([[("username", "user1").into()]])
            .ids(1)
            .query(),
        1,
    );
}

#[test]
fn update_indexed_value() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("username").query(), 0);
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("username", "user1").into()]])
            .query(),
        1,
    );
    db.exec_mut(
        QueryBuilder::insert()
            .values([[("username", "user2").into()]])
            .ids(1)
            .query(),
        1,
    );
}

#[test]
fn update_indexed_value_rollback() {
    let mut db = TestDb::new();
    db.exec_mut(QueryBuilder::insert().index("username").query(), 0);
    db.exec_mut(
        QueryBuilder::insert()
            .nodes()
            .values([[("username", "user1").into()]])
            .query(),
        1,
    );
    db.transaction_mut_error(
        |t| -> Result<(), QueryError> {
            t.exec_mut(
                QueryBuilder::insert()
                    .values([[("username", "user2").into()]])
                    .ids(1)
                    .query(),
            )?;
            Err(QueryError::from("error"))
        },
        QueryError::from("error"),
    );
    db.exec(
        QueryBuilder::search()
            .index("username")
            .value("user1")
            .query(),
        1,
    );
}
